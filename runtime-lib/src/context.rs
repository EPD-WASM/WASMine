use ir::structs::data::DataMode;
use ir::structs::export::ExportDesc;
use ir::structs::import::ImportDesc;
use ir::structs::value::Value;
use runtime_interface::RTImport;
use std::collections::HashMap;
use std::path::PathBuf;
use wasm_types::FuncType;

#[derive(Clone)]
pub struct RTMemory {
    pub min_size: u32,
    pub max_size: u32,
    pub shared: bool,
}

pub struct RTTable {
    pub min_size: u32,
    pub max_size: u32,
    pub r#type: RTTableType,
}

#[derive(PartialEq, Eq)]
pub enum RTTableType {
    FuncRef,
    ExternRef,
}

pub enum RTDataType {
    /// Active
    PreLoad { memory: u32, offset: Value },
    /// Passive
    RuntimeLoad,
}

pub enum RTDataSource {
    /// In-memory data for small data segments
    Inline(*const u8),
    /// File-backed data for large data segments (mmaped to memory)
    File(PathBuf),
}

pub struct RTData {
    pub r#type: RTDataType,
    pub source: RTDataSource,
}

pub struct RTExport {
    pub function_idx: u32,
    pub function_type_idx: u32,
}

pub struct RTFunction {
    pub type_idx: u32,
    // function pointer, function may take more params
    pub func: Box<dyn Fn()>,
}

#[derive(Default)]
pub struct RTContext {
    /// Tables to be initialized
    pub tables: Vec<RTTable>,

    /// Memories to be initialized
    pub memories: Vec<RTMemory>,
    pub datas: Vec<RTData>,

    pub imports: Vec<RTImport>,

    pub exported_functions: HashMap<String, RTExport>,

    // raw parsed data
    pub module: Module,
}

use crate::error::RuntimeError;
use crate::{wasi, WASM_PAGE_LIMIT};
use ir::structs::module::Module;
use wasm_types::RefType;

fn collect_available_imports() -> HashMap<&'static str, RTImport> {
    wasi::collect_available_imports()
}

impl RTContext {
    pub fn new(module: Module) -> Result<Self, RuntimeError> {
        let mut config = RTContext::default();
        for table in module.tables.iter() {
            config.tables.push(RTTable {
                min_size: table.r#type.lim.min,
                max_size: table.r#type.lim.max.unwrap_or(WASM_PAGE_LIMIT),
                r#type: match table.r#type.ref_type {
                    RefType::FunctionReference => RTTableType::FuncRef,
                    RefType::ExternReference => RTTableType::ExternRef,
                },
            })
        }
        for data in module.datas.iter() {
            config.datas.push(RTData {
                r#type: match data.mode.clone() {
                    DataMode::Active { memory, offset } => RTDataType::PreLoad { memory, offset },
                    DataMode::Passive => RTDataType::RuntimeLoad,
                },
                source: RTDataSource::Inline(data.init.as_ptr()),
            })
        }

        for export in module.exports.iter() {
            if let ExportDesc::Func(func_idx) = export.desc {
                config.exported_functions.insert(
                    export.name.clone(),
                    RTExport {
                        function_idx: func_idx,
                        function_type_idx: module.ir.functions[func_idx as usize].type_idx,
                    },
                );
            } else {
                log::warn!("Export type {:?} not implemented", export.desc);
            }
        }

        let available_imports = collect_available_imports();
        for import in module.imports.iter() {
            if let ImportDesc::Func(type_idx) = import.desc {
                let requested_func_name = format!("{}.{}", import.module, import.name);
                let requested_func_type = module.function_types[type_idx as usize].clone();
                let available_import = match available_imports.get(requested_func_name.as_str()) {
                    Some(import) => import,
                    None => {
                        return Err(RuntimeError::InvalidImport(format!(
                            "Import not found: {}",
                            requested_func_name
                        )))
                    }
                };
                if requested_func_type != available_import.function_type {
                    return Err(RuntimeError::InvalidImport(format!(
                        "Import type mismatch for function {}. WASM module expected {:?}, our runtime offers {:?}.",
                        requested_func_name, requested_func_type, available_import.function_type
                    )));
                }
                config.imports.push(available_import.clone());
            } else {
                return Err(RuntimeError::NotImplemented(format!(
                    "Import type {:?}",
                    import.desc
                )));
            }
        }
        for memory in module.memories.iter() {
            config.memories.push(RTMemory {
                min_size: memory.limits.min,
                max_size: memory.limits.max.unwrap_or(WASM_PAGE_LIMIT),
                shared: false,
            })
        }

        config.module = module;
        Ok(config)
    }

    pub(crate) fn query_start_function(&self) -> Result<u32, RuntimeError> {
        if let Some(start_func_idx) = self.module.entry_point {
            return Ok(start_func_idx);
        }
        self.find_func("_start")
            .or_else(|_| self.find_func("run"))
            .or(Err(RuntimeError::NoStartFunction))
    }

    pub fn find_func(&self, name: &str) -> Result<u32, RuntimeError> {
        if let Some(f) = self.exported_functions.get(name) {
            return Ok(f.function_idx);
        }
        Err(RuntimeError::NoStartFunction)
    }

    pub fn get_function_type(&self, func_idx: u32) -> &FuncType {
        &self.module.function_types[self.module.ir.functions[func_idx as usize].type_idx as usize]
    }
}
