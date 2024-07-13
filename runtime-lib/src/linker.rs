use crate::{
    engine::{Engine, EngineError},
    instance_handle::{InstanceHandle, InstantiationError},
    segmented_list::SegmentedList,
    tables::TableItem,
    wasi, Cluster,
};
use ir::structs::{export::ExportDesc, module::Module as WasmModule};
use runtime_interface::{ExecutionContext, RawFunctionPtr};
use std::{collections::HashMap, rc::Rc};
use wasm_types::{FuncType, GlobalIdx, GlobalType, ImportDesc, Limits, MemType, TableType};

#[derive(thiserror::Error, Debug)]
pub enum LinkingError {
    #[error("Module cluster mismatch. Bound linker received module from foreign cluster.")]
    ClusterMismatch,
    #[error("Missing required module '{module_name}'.")]
    ModuleNotFound { module_name: String },
    #[error("Function type mismatch. Requested: {requested}, Actual: {actual}.")]
    FunctionTypeMismatch {
        requested: FuncType,
        actual: FuncType,
    },
    #[error("Global type mismatch. Requested: {requested:?}, Actual: {actual:?}.")]
    GlobalTypeMismatch {
        requested: GlobalType,
        actual: GlobalType,
    },
    #[error("Table type mismatch. Requested: {requested:?}, Actual: {actual:?}.")]
    TableTypeMismatch {
        requested: TableType,
        actual: TableType,
    },
    #[error("Memory type mismatch. Requested: {requested:?}, Actual: {actual:?}.")]
    MemoryTypeMismatch { requested: MemType, actual: MemType },
    #[error("Global '{global_name}' not found in module '{module_name}'.")]
    GlobalNotFound {
        global_name: String,
        module_name: String,
    },
    #[error("Memory '{memory_name}' not found in module '{module_name}'.")]
    MemoryNotFound {
        memory_name: String,
        module_name: String,
    },
    #[error("Encountered error during call to execution engine: {0}")]
    EngineError(#[from] EngineError),
    #[error("Function '{name}' not found in module '{module_name}'.")]
    FunctionNotFound { module_name: String, name: String },
    #[error("Error during instantiation of module: {0}")]
    InstantiationError(#[from] InstantiationError),
}

pub struct BoundLinker<'a> {
    registered_modules: HashMap<String, &'a InstanceHandle<'a>>,
    transferred_handles: SegmentedList<InstanceHandle<'a>>,
    cluster: &'a Cluster,
}

impl<'a> BoundLinker<'a> {
    pub fn new(cluster: &'a Cluster) -> Self {
        Self {
            registered_modules: Default::default(),
            transferred_handles: Default::default(),
            cluster,
        }
    }

    pub fn register(
        &mut self,
        module_name: &str,
        instance_handle: &'a InstanceHandle<'a>,
    ) -> Result<(), LinkingError> {
        if instance_handle.cluster.uuid != self.cluster.uuid {
            return Err(LinkingError::ClusterMismatch);
        }
        self.registered_modules
            .insert(module_name.to_owned(), instance_handle);
        Ok(())
    }

    pub fn transfer(
        &mut self,
        module_name: &str,
        instance_handle: InstanceHandle<'a>,
    ) -> Result<(), LinkingError> {
        if instance_handle.cluster.uuid != self.cluster.uuid {
            return Err(LinkingError::ClusterMismatch);
        }
        self.transferred_handles.push(instance_handle);
        let reference = self
            .transferred_handles
            .get_last_segments_ref()
            .first()
            .unwrap();
        self.registered_modules
            .insert(module_name.to_owned(), reference);
        Ok(())
    }

    pub fn instantiate_and_link(
        &self,
        module: Rc<WasmModule>,
        engine: Engine,
    ) -> Result<InstanceHandle<'a>, LinkingError> {
        let imports = self.collect_imports_for_module(&module)?;
        Ok(InstanceHandle::new(self.cluster, module, engine, imports)?)
    }

    fn collect_imports_for_module(
        &self,
        module: &WasmModule,
    ) -> Result<RTImportCollection, LinkingError> {
        let mut imports = RTImportCollection::default();

        for import in module.imports.iter() {
            let exporting_module = match self.registered_modules.get(&import.module) {
                Some(m) => m,
                None => {
                    return Err(LinkingError::ModuleNotFound {
                        module_name: import.module.clone(),
                    })
                }
            };

            match &import.desc {
                ImportDesc::Func(type_idx) => {
                    let requested_function_type = module.function_types[*type_idx as usize].clone();
                    let actual_function_type =
                        match exporting_module.get_function_type_from_name(&import.name) {
                            Some(t) => t,
                            None => {
                                return Err(LinkingError::FunctionNotFound {
                                    name: import.name.clone(),
                                    module_name: import.module.clone(),
                                })
                            }
                        };
                    if requested_function_type != *actual_function_type {
                        return Err(LinkingError::FunctionTypeMismatch {
                            requested: requested_function_type,
                            actual: actual_function_type.clone(),
                        });
                    }
                    imports.functions.push(RTFuncImport {
                        name: format!("{}.{}", import.module, import.name),
                        function_type: requested_function_type,
                        callable: exporting_module.get_raw_function_ptr(&import.name)?,
                        execution_context: Some(exporting_module.execution_context()),
                    });
                }
                ImportDesc::Global((requested_type, idx)) => {
                    let exported_global_idx = match exporting_module
                        .wasm_module()
                        .exports
                        .iter()
                        .find_map(|export| match &export.desc {
                            ExportDesc::Global(idx) if export.name == import.name => Some(idx),
                            _ => None,
                        }) {
                        Some(idx) => idx,
                        None => {
                            return Err(LinkingError::GlobalNotFound {
                                global_name: import.name.clone(),
                                module_name: import.module.clone(),
                            })
                        }
                    };
                    let exported_global =
                        &exporting_module.wasm_module().globals[*exported_global_idx as usize];
                    let actual_type = &exported_global.r#type;
                    if requested_type != actual_type {
                        return Err(LinkingError::GlobalTypeMismatch {
                            requested: *requested_type,
                            actual: *actual_type,
                        });
                    }
                    imports.globals.push(RTGlobalImport {
                        name: import.name.clone(),
                        addr: exporting_module.globals(*exported_global_idx).addr,
                        r#type: *requested_type,
                        idx: *idx,
                    });
                }
                ImportDesc::Mem(expected_limits) => {
                    let exported_memory_idx = match exporting_module
                        .wasm_module()
                        .exports
                        .iter()
                        .find_map(|export| match &export.desc {
                            ExportDesc::Mem(idx) if export.name == import.name => Some(idx),
                            _ => None,
                        }) {
                        Some(idx) => idx,
                        None => {
                            return Err(LinkingError::MemoryNotFound {
                                memory_name: import.name.clone(),
                                module_name: import.module.clone(),
                            })
                        }
                    };
                    let exported_memory =
                        &exporting_module.wasm_module().memories[*exported_memory_idx as usize];

                    let mut actual_limits = exported_memory.limits;
                    actual_limits.min = actual_limits
                        .min
                        .max(exporting_module.memories(*exported_memory_idx).0.size);

                    let max_actual_len = actual_limits.max.unwrap_or(u32::MAX);
                    let max_expected_len = expected_limits.max.unwrap_or(u32::MAX);
                    if expected_limits.min <= actual_limits.min
                        && max_expected_len >= max_actual_len
                    {
                        /* Good, these are exactly the qualities we need :) */
                    } else {
                        return Err(LinkingError::MemoryTypeMismatch {
                            requested: *expected_limits,
                            actual: actual_limits,
                        });
                    }
                    imports.memories.push(RTMemoryImport {
                        name: import.name.clone(),
                        limits: actual_limits,
                    });
                }
                ImportDesc::Table(expected_type) => {
                    let exported_table_idx = match exporting_module
                        .wasm_module()
                        .exports
                        .iter()
                        .find_map(|export| match &export.desc {
                            ExportDesc::Table(idx) if export.name == import.name => Some(idx),
                            _ => None,
                        }) {
                        Some(idx) => idx,
                        None => {
                            return Err(LinkingError::GlobalNotFound {
                                global_name: import.name.clone(),
                                module_name: import.module.clone(),
                            })
                        }
                    };
                    let exported_table =
                        &exporting_module.wasm_module().tables[*exported_table_idx as usize];
                    let actual_type = &exported_table.r#type;

                    let max_expected_len = expected_type.lim.max.unwrap_or(u32::MAX);
                    let max_actual_len = actual_type.lim.max.unwrap_or(u32::MAX);
                    if expected_type.ref_type == actual_type.ref_type
                        && expected_type.lim.min <= actual_type.lim.min
                        && max_expected_len >= max_actual_len
                    {
                        /* Good, these are exactly the qualities we need :) */
                    } else {
                        return Err(LinkingError::TableTypeMismatch {
                            requested: *expected_type,
                            actual: *actual_type,
                        });
                    }
                    let vals = &exporting_module.tables(*exported_table_idx).values;
                    let vals_ref = *vals as *const _ as *mut _;
                    imports.tables.push(RTTableImport {
                        name: import.name.clone(),
                        instance_ref: vals_ref,
                        r#type: *expected_type,
                    });
                }
            }
        }
        Ok(imports)
    }
}

#[derive(Default)]
pub struct Linker {
    host_functions: HashMap<String, RTFuncImport>,
}

impl Linker {
    pub fn bind_to<'a>(&self, cluster: &'a Cluster) -> BoundLinker<'a> {
        BoundLinker::new(cluster)
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn link_wasi(&mut self) {
        self.host_functions.extend(
            wasi::collect_available_imports()
                .into_iter()
                .map(|(k, v)| (k.into(), v))
                .collect::<HashMap<_, _>>(),
        );
    }
}

#[allow(clippy::fn_to_numeric_cast)]
pub(crate) fn rt_func_imports() -> HashMap<&'static str, RTFuncImport> {
    let mut imports = HashMap::new();
    imports.insert(
        "memory_grow",
        RTFuncImport {
            name: "memory_grow".into(),
            function_type: FuncType(Vec::new(), Vec::new()),
            callable: runtime_interface::memory_grow as RawFunctionPtr,
            execution_context: None,
        },
    );
    imports.insert(
        "memory_fill",
        RTFuncImport {
            name: "memory_fill".into(),
            function_type: FuncType(Vec::new(), Vec::new()),
            callable: runtime_interface::memory_fill as RawFunctionPtr,
            execution_context: None,
        },
    );
    imports.insert(
        "memory_copy",
        RTFuncImport {
            name: "memory_copy".into(),
            function_type: FuncType(Vec::new(), Vec::new()),
            callable: runtime_interface::memory_copy as RawFunctionPtr,
            execution_context: None,
        },
    );
    imports.insert(
        "memory_init",
        RTFuncImport {
            name: "memory_init".into(),
            function_type: FuncType(Vec::new(), Vec::new()),
            callable: runtime_interface::memory_init as RawFunctionPtr,
            execution_context: None,
        },
    );
    imports.insert(
        "data_drop",
        RTFuncImport {
            name: "data_drop".into(),
            function_type: FuncType(Vec::new(), Vec::new()),
            callable: runtime_interface::data_drop as RawFunctionPtr,
            execution_context: None,
        },
    );
    imports.insert(
        "indirect_call",
        RTFuncImport {
            name: "indirect_call".into(),
            function_type: FuncType(Vec::new(), Vec::new()),
            callable: runtime_interface::indirect_call as RawFunctionPtr,
            execution_context: None,
        },
    );
    imports.insert(
        "table_set",
        RTFuncImport {
            name: "table_set".into(),
            function_type: FuncType(Vec::new(), Vec::new()),
            callable: runtime_interface::table_set as RawFunctionPtr,
            execution_context: None,
        },
    );
    imports.insert(
        "table_get",
        RTFuncImport {
            name: "table_get".into(),
            function_type: FuncType(Vec::new(), Vec::new()),
            callable: runtime_interface::table_get as RawFunctionPtr,
            execution_context: None,
        },
    );
    imports.insert(
        "table_grow",
        RTFuncImport {
            name: "table_grow".into(),
            function_type: FuncType(Vec::new(), Vec::new()),
            callable: runtime_interface::table_grow as RawFunctionPtr,
            execution_context: None,
        },
    );
    imports.insert(
        "table_size",
        RTFuncImport {
            name: "table_size".into(),
            function_type: FuncType(Vec::new(), Vec::new()),
            callable: runtime_interface::table_size as RawFunctionPtr,
            execution_context: None,
        },
    );
    imports.insert(
        "table_fill",
        RTFuncImport {
            name: "table_fill".into(),
            function_type: FuncType(Vec::new(), Vec::new()),
            callable: runtime_interface::table_fill as RawFunctionPtr,
            execution_context: None,
        },
    );
    imports.insert(
        "table_copy",
        RTFuncImport {
            name: "table_copy".into(),
            function_type: FuncType(Vec::new(), Vec::new()),
            callable: runtime_interface::table_copy as RawFunctionPtr,
            execution_context: None,
        },
    );
    imports.insert(
        "table_init",
        RTFuncImport {
            name: "table_init".into(),
            function_type: FuncType(Vec::new(), Vec::new()),
            callable: runtime_interface::table_init as RawFunctionPtr,
            execution_context: None,
        },
    );
    imports.insert(
        "elem_drop",
        RTFuncImport {
            name: "elem_drop".into(),
            function_type: FuncType(Vec::new(), Vec::new()),
            callable: runtime_interface::elem_drop as RawFunctionPtr,
            execution_context: None,
        },
    );
    imports
}

#[derive(Clone, Debug)]
pub(crate) struct RTFuncImport {
    pub(crate) name: String,
    pub(crate) function_type: FuncType,
    pub(crate) callable: RawFunctionPtr,
    // required, because function imports are basically closures over all module state. This is not provided by host functions.
    pub(crate) execution_context: Option<*mut ExecutionContext>,
}

#[derive(Clone, Debug)]
pub(crate) struct RTMemoryImport {
    pub(crate) name: String,
    pub(crate) limits: Limits,
}

#[derive(Clone)]
pub(crate) struct RTGlobalImport {
    pub(crate) name: String,
    pub(crate) addr: *mut u64,
    pub(crate) r#type: GlobalType,
    pub(crate) idx: GlobalIdx,
}

#[derive(Clone)]
pub(crate) struct RTTableImport {
    pub(crate) name: String,
    pub(crate) instance_ref: *mut Vec<TableItem>,
    pub(crate) r#type: TableType,
}

#[derive(Default)]
pub(crate) struct RTImportCollection {
    pub(crate) functions: Vec<RTFuncImport>,
    pub(crate) globals: Vec<RTGlobalImport>,
    pub(crate) memories: Vec<RTMemoryImport>,
    pub(crate) tables: Vec<RTTableImport>,
}
