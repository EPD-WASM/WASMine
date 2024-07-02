use crate::{
    engine::Engine, error::RuntimeError, module_instance::WasmModuleInstance, wasi, RTFuncImport,
    RTGlobalImport, RTMemoryImport, RTTableImport,
};
use ir::structs::module::Module as WasmModule;
use runtime_interface::RawFunctionPtr;
use std::{collections::HashMap, rc::Rc};
use wasm_types::{FuncType, ImportDesc};

#[derive(Default)]
pub struct Linker {
    func_storage: HashMap<String, RTFuncImport>,
    memory_storage: HashMap<String, RTMemoryImport>,
    global_storage: HashMap<String, RTGlobalImport>,
    table_storage: HashMap<String, RTTableImport>,
}

impl Linker {
    pub fn new() -> Self {
        let mut s = Self::default();
        // TODO: make this more efficient
        for (_, import) in collect_available_imports() {
            s.register_external_func(import)
        }
        s
    }

    pub fn register_external_func(&mut self, import: RTFuncImport) {
        self.func_storage.insert(import.name.to_owned(), import);
    }

    fn lookup_func_addr(&self, fn_name: &str) -> Option<RawFunctionPtr> {
        self.func_storage.get(fn_name).map(|res| res.callable)
    }

    fn lookup_func_type(&self, fn_name: &str) -> Option<FuncType> {
        self.func_storage
            .get(fn_name)
            .map(|res| res.function_type.clone())
    }

    pub fn register_external_memory(&mut self, import: RTMemoryImport) {
        self.memory_storage.insert(import.name.to_owned(), import);
    }

    fn lookup_memory(&self, name: &str) -> Option<&RTMemoryImport> {
        self.memory_storage.get(name)
    }

    pub fn register_external_table(&mut self, import: RTTableImport) {
        self.table_storage.insert(import.name.to_owned(), import);
    }

    fn lookup_table(&mut self, name: &str) -> Option<&RTTableImport> {
        self.table_storage.get(name)
    }

    pub fn register_external_global(&mut self, import: RTGlobalImport) {
        self.global_storage.insert(import.name.to_owned(), import);
    }

    fn lookup_global(&mut self, name: &str) -> Option<&RTGlobalImport> {
        self.global_storage.get(name)
    }

    pub fn link(
        mut self,
        wasm_module: Rc<WasmModule>,
        engine: Engine,
    ) -> Result<WasmModuleInstance, RuntimeError> {
        let mut module_instance = WasmModuleInstance::new(wasm_module.clone(), engine);

        for import in wasm_module.imports.iter() {
            let requested_import_name = format!("{}.{}", import.module, import.name);
            match import.desc {
                ImportDesc::Func(type_idx) => {
                    let requested_func_type = wasm_module.function_types[type_idx as usize].clone();
                    match self.lookup_func_type(requested_import_name.as_str()) {
                    Some(t) if t == requested_func_type => module_instance.add_func_import(&requested_import_name, self.lookup_func_addr(requested_import_name.as_str()).unwrap()),
                    Some(t) => {return Err(RuntimeError::InvalidImport(format!("Supplied import signature {:?} does not match declared import signature {:?}.", t, requested_func_type)))},
                    None => log::warn!("Could not validate import signature for imported function '{}'", requested_import_name)
                };
                }
                ImportDesc::Mem(limits) => {
                    match self.lookup_memory(requested_import_name.as_str()) {
                    Some(mem) if mem.limits == limits => {
                        module_instance.add_memory_import(mem.instance.clone())
                    },
                    Some(mem) => return Err(RuntimeError::InvalidImport(format!("Supplied import memory limits {:?} do not match declared import memory limits {:?}.", mem, limits))),
                    None => return Err(RuntimeError::InvalidImport(format!("Could not find memory import '{}'", requested_import_name)))
                };
                }
                ImportDesc::Global(t) => {
                    match self.lookup_global(&requested_import_name) {
                        Some(glob) if glob.r#type == t => {
                            module_instance.add_global_import(glob.instance.clone())
                        },
                        Some(glob) => return Err(RuntimeError::InvalidImport(format!("Supplied import global type {:?} does not match declared import global type {:?}.", glob.r#type, t))),
                        None => return Err(RuntimeError::InvalidImport(format!("Could not find global import '{}'", requested_import_name)))
                    }
                },
                ImportDesc::Table(t) => {
                    match self.lookup_table(&requested_import_name) {
                        Some(table) if table.r#type == t => {
                            module_instance.add_table_import(table.instance.clone())
                        },
                        Some(table) => return Err(RuntimeError::InvalidImport(format!("Supplied import table type {:?} does not match declared import table type {:?}.", table.r#type, t))),
                        None => return Err(RuntimeError::InvalidImport(format!("Could not find table import '{}'", requested_import_name)))
                    }
                }
            }
        }

        // add rt helper functions (don't need to be explicitly imported)
        for (_, import) in collect_available_imports() {
            module_instance.add_func_import(&import.name, import.callable)
        }

        Ok(module_instance)
    }
}

#[allow(clippy::fn_to_numeric_cast)]
fn collect_available_imports() -> HashMap<&'static str, RTFuncImport> {
    let mut imports = wasi::collect_available_imports();
    imports.insert(
        "memory_grow",
        RTFuncImport {
            name: "memory_grow".into(),
            function_type: (Vec::new(), Vec::new()),
            callable: runtime_interface::memory_grow as RawFunctionPtr,
        },
    );
    imports.insert(
        "memory_fill",
        RTFuncImport {
            name: "memory_fill".into(),
            function_type: (Vec::new(), Vec::new()),
            callable: runtime_interface::memory_fill as RawFunctionPtr,
        },
    );
    imports.insert(
        "memory_copy",
        RTFuncImport {
            name: "memory_copy".into(),
            function_type: (Vec::new(), Vec::new()),
            callable: runtime_interface::memory_copy as RawFunctionPtr,
        },
    );
    imports
}
