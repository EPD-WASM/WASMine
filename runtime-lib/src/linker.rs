use crate::{
    helper::segmented_list::SegmentedList,
    objects::{
        engine::EngineError,
        functions::{Function, IntoFunc},
        instance_handle::{InstanceHandle, InstantiationError},
        tables::TableObject,
    },
    wasi::{WasiContext, WASI_FUNCS},
    Cluster, Engine,
};
use ir::structs::{module::Module as WasmModule, value::ValueRaw};
use once_cell::sync::Lazy;
use runtime_interface::{ExecutionContext, RawFunctionPtr};
use std::{collections::HashMap, rc::Rc};
use wasm_types::{FuncType, GlobalIdx, GlobalType, ImportDesc, Limits, MemType, TableType};

#[derive(thiserror::Error, Debug)]
pub enum LinkingError {
    #[error("Module cluster mismatch. Bound linker received module from foreign cluster.")]
    ClusterMismatch,
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
    #[error("Function '{name}' not found from module '{module_name}'.")]
    FunctionNotFound { module_name: String, name: String },
    #[error("Error during instantiation of module: {0}")]
    InstantiationError(#[from] InstantiationError),
}

pub struct BoundLinker<'a> {
    registered_modules: HashMap<String, &'a InstanceHandle<'a>>,
    transferred_handles: SegmentedList<InstanceHandle<'a>>,
    host_functions: HostFunctionStorage,
    cluster: &'a Cluster,
}

impl<'a> BoundLinker<'a> {
    pub fn new(cluster: &'a Cluster) -> Self {
        Self {
            registered_modules: Default::default(),
            transferred_handles: Default::default(),
            host_functions: HostFunctionStorage::new(),
            cluster,
        }
    }

    fn new_from_linker(cluster: &'a Cluster, host_functions: HostFunctionStorage) -> Self {
        Self {
            registered_modules: Default::default(),
            transferred_handles: Default::default(),
            host_functions,
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
        Ok(InstanceHandle::new(
            self.cluster,
            module,
            engine,
            imports,
            None,
        )?)
    }

    pub fn instantiate_and_link_with_wasi(
        &self,
        module: Rc<WasmModule>,
        engine: Engine,
        wasi_ctxt: WasiContext,
    ) -> Result<InstanceHandle<'a>, LinkingError> {
        let imports = self.collect_imports_for_module(&module)?;
        Ok(InstanceHandle::new(
            self.cluster,
            module,
            engine,
            imports,
            Some(wasi_ctxt),
        )?)
    }

    fn collect_imports_for_module(
        &self,
        module: &WasmModule,
    ) -> Result<DependencyStore, LinkingError> {
        let mut imports = DependencyStore::default();

        for import in module.imports.iter() {
            let name = DependencyName {
                module: import.module.clone(),
                name: import.name.clone(),
            };

            // filter out host & wasi function imports early
            if matches!(import.desc, ImportDesc::Func(_)) {
                #[allow(clippy::borrow_interior_mutable_const)]
                if WASI_FUNCS.contains(&(import.module.as_str(), import.name.as_str())) {
                    imports.functions.push(FunctionDependency {
                        name,
                        func: self.cluster.alloc_function(WasiContext::get_func_by_name(
                            &import.module,
                            &import.name,
                        )),
                    });
                    continue;
                }
                if let Some(host_func_import) =
                    self.host_functions.get(&import.module, &import.name)
                {
                    imports.functions.push(FunctionDependency {
                        name,
                        func: host_func_import,
                    });
                    continue;
                }
            }

            let exporting_module = match self.registered_modules.get(&import.module) {
                Some(m) => m,
                None => {
                    return Err(LinkingError::FunctionNotFound {
                        module_name: import.module.clone(),
                        name: import.name.clone(),
                    })
                }
            };

            match &import.desc {
                ImportDesc::Func(type_idx) => {
                    let requested_function_type = module.function_types[*type_idx as usize];
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
                    if requested_function_type != actual_function_type {
                        return Err(LinkingError::FunctionTypeMismatch {
                            requested: requested_function_type,
                            actual: actual_function_type,
                        });
                    }

                    let import_func_idx = exporting_module
                        .wasm_module()
                        .exports
                        .find_function_idx(&import.name)
                        .unwrap();
                    imports.functions.push(FunctionDependency {
                        name,
                        func: match exporting_module.get_export_by_name(&import.name) {
                            Ok(f) => f,
                            Err(e) => {
                                return Err(LinkingError::FunctionNotFound {
                                    module_name: import.module.clone(),
                                    name: import.name.clone(),
                                });
                            }
                        },
                    });
                }
                ImportDesc::Global((requested_type, idx)) => {
                    let exported_global_idx = match exporting_module
                        .wasm_module()
                        .exports
                        .find_global_idx(&import.name)
                    {
                        Some(idx) => idx,
                        None => {
                            return Err(LinkingError::GlobalNotFound {
                                global_name: import.name.clone(),
                                module_name: import.module.clone(),
                            })
                        }
                    };
                    let exported_global =
                        &exporting_module.wasm_module().globals[exported_global_idx as usize];
                    let actual_type = &exported_global.r#type;
                    if requested_type != actual_type {
                        return Err(LinkingError::GlobalTypeMismatch {
                            requested: *requested_type,
                            actual: *actual_type,
                        });
                    }
                    imports.globals.push(RTGlobalImport {
                        addr: exporting_module.globals(exported_global_idx).addr,
                        r#type: *requested_type,
                        idx: *idx,
                    });
                }
                ImportDesc::Mem(expected_limits) => {
                    let exported_memory_idx = match exporting_module
                        .wasm_module()
                        .exports
                        .find_memory_idx(&import.name)
                    {
                        Some(idx) => idx,
                        None => {
                            return Err(LinkingError::MemoryNotFound {
                                memory_name: import.name.clone(),
                                module_name: import.module.clone(),
                            })
                        }
                    };
                    let exported_memory =
                        &exporting_module.wasm_module().memories[exported_memory_idx as usize];

                    let mut actual_limits = exported_memory.limits;
                    actual_limits.min = actual_limits
                        .min
                        .max(exporting_module.memories(exported_memory_idx).0.size);

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
                        .find_table_idx(&import.name)
                    {
                        Some(idx) => idx,
                        None => {
                            return Err(LinkingError::GlobalNotFound {
                                global_name: import.name.clone(),
                                module_name: import.module.clone(),
                            })
                        }
                    };
                    let exported_table =
                        &exporting_module.wasm_module().tables[exported_table_idx as usize];
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
                    let vals = &exporting_module.tables(exported_table_idx).values;
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

    pub fn link_host_function<F, Params, Returns>(
        &mut self,
        module_name: &str,
        function_name: &str,
        callable: F,
    ) where
        F: IntoFunc<Params, Returns>,
    {
        self.host_functions
            .insert(module_name, function_name, callable);
    }
}

#[derive(Default)]
pub struct Linker {
    host_functions: HostFunctionStorage,
}

impl Linker {
    pub fn bind_to<'a>(&self, cluster: &'a Cluster) -> BoundLinker<'a> {
        BoundLinker::new_from_linker(cluster, self.host_functions.clone())
    }

    pub fn new() -> Self {
        Self::default()
    }

    pub fn link_host_function<F, Params, Returns>(
        &mut self,
        module_name: &str,
        function_name: &str,
        callable: F,
    ) where
        F: IntoFunc<Params, Returns>,
    {
        self.host_functions
            .insert(module_name, function_name, callable);
    }
}

static MEMORY_GROW_RT_FUNC: Lazy<Function> = Lazy::new(|| {
    Function::from_runtime_func(unsafe {
        RawFunctionPtr::new_unchecked(runtime_interface::memory_grow as _)
    })
});

static MEMORY_COPY_RT_FUNC: Lazy<Function> = Lazy::new(|| {
    Function::from_runtime_func(unsafe {
        RawFunctionPtr::new_unchecked(runtime_interface::memory_copy as _)
    })
});

static MEMORY_INIT_RT_FUNC: Lazy<Function> = Lazy::new(|| {
    Function::from_runtime_func(unsafe {
        RawFunctionPtr::new_unchecked(runtime_interface::memory_init as _)
    })
});

static MEMORY_FILL_RT_FUNC: Lazy<Function> = Lazy::new(|| {
    Function::from_runtime_func(unsafe {
        RawFunctionPtr::new_unchecked(runtime_interface::memory_fill as _)
    })
});

static DATA_DROP_RT_FUNC: Lazy<Function> = Lazy::new(|| {
    Function::from_runtime_func(unsafe {
        RawFunctionPtr::new_unchecked(runtime_interface::data_drop as _)
    })
});

static INDIRECT_CALL_RT_FUNC: Lazy<Function> = Lazy::new(|| {
    Function::from_runtime_func(unsafe {
        RawFunctionPtr::new_unchecked(runtime_interface::indirect_call as _)
    })
});

static TABLE_SET_RT_FUNC: Lazy<Function> = Lazy::new(|| {
    Function::from_runtime_func(unsafe {
        RawFunctionPtr::new_unchecked(runtime_interface::table_set as _)
    })
});

static TABLE_GET_RT_FUNC: Lazy<Function> = Lazy::new(|| {
    Function::from_runtime_func(unsafe {
        RawFunctionPtr::new_unchecked(runtime_interface::table_get as _)
    })
});

static TABLE_GROW_RT_FUNC: Lazy<Function> = Lazy::new(|| {
    Function::from_runtime_func(unsafe {
        RawFunctionPtr::new_unchecked(runtime_interface::table_grow as _)
    })
});

static TABLE_SIZE_RT_FUNC: Lazy<Function> = Lazy::new(|| {
    Function::from_runtime_func(unsafe {
        RawFunctionPtr::new_unchecked(runtime_interface::table_size as _)
    })
});

static TABLE_FILL_RT_FUNC: Lazy<Function> = Lazy::new(|| {
    Function::from_runtime_func(unsafe {
        RawFunctionPtr::new_unchecked(runtime_interface::table_fill as _)
    })
});

static TABLE_COPY_RT_FUNC: Lazy<Function> = Lazy::new(|| {
    Function::from_runtime_func(unsafe {
        RawFunctionPtr::new_unchecked(runtime_interface::table_copy as _)
    })
});

static TABLE_INIT_RT_FUNC: Lazy<Function> = Lazy::new(|| {
    Function::from_runtime_func(unsafe {
        RawFunctionPtr::new_unchecked(runtime_interface::table_init as _)
    })
});

static ELEM_DROP_RT_FUNC: Lazy<Function> = Lazy::new(|| {
    Function::from_runtime_func(unsafe {
        RawFunctionPtr::new_unchecked(runtime_interface::elem_drop as _)
    })
});

#[allow(clippy::fn_to_numeric_cast)]
pub(crate) fn rt_func_imports(
    execution_context: *mut ExecutionContext,
) -> Vec<FunctionDependency<'static>> {
    let module_name = "__wasmine_runtime".to_string();
    vec![
        FunctionDependency {
            name: DependencyName {
                module: module_name.clone(),
                name: "memory_fill".to_string(),
            },
            func: &MEMORY_FILL_RT_FUNC,
        },
        FunctionDependency {
            name: DependencyName {
                module: module_name.clone(),
                name: "memory_copy".to_string(),
            },
            func: &MEMORY_COPY_RT_FUNC,
        },
        FunctionDependency {
            name: DependencyName {
                module: module_name.clone(),
                name: "memory_init".to_string(),
            },
            func: &MEMORY_INIT_RT_FUNC,
        },
        FunctionDependency {
            name: DependencyName {
                module: module_name.clone(),
                name: "memory_grow".to_string(),
            },
            func: &MEMORY_GROW_RT_FUNC,
        },
        FunctionDependency {
            name: DependencyName {
                module: module_name.clone(),
                name: "data_drop".to_string(),
            },
            func: &DATA_DROP_RT_FUNC,
        },
        FunctionDependency {
            name: DependencyName {
                module: module_name.clone(),
                name: "indirect_call".to_string(),
            },
            func: &INDIRECT_CALL_RT_FUNC,
        },
        FunctionDependency {
            name: DependencyName {
                module: module_name.clone(),
                name: "table_set".to_string(),
            },
            func: &TABLE_SET_RT_FUNC,
        },
        FunctionDependency {
            name: DependencyName {
                module: module_name.clone(),
                name: "table_get".to_string(),
            },
            func: &TABLE_GET_RT_FUNC,
        },
        FunctionDependency {
            name: DependencyName {
                module: module_name.clone(),
                name: "table_grow".to_string(),
            },
            func: &TABLE_GROW_RT_FUNC,
        },
        FunctionDependency {
            name: DependencyName {
                module: module_name.clone(),
                name: "table_size".to_string(),
            },
            func: &TABLE_SIZE_RT_FUNC,
        },
        FunctionDependency {
            name: DependencyName {
                module: module_name.clone(),
                name: "table_fill".to_string(),
            },
            func: &TABLE_FILL_RT_FUNC,
        },
        FunctionDependency {
            name: DependencyName {
                module: module_name.clone(),
                name: "table_copy".to_string(),
            },
            func: &TABLE_COPY_RT_FUNC,
        },
        FunctionDependency {
            name: DependencyName {
                module: module_name.clone(),
                name: "table_init".to_string(),
            },
            func: &TABLE_INIT_RT_FUNC,
        },
        FunctionDependency {
            name: DependencyName {
                module: module_name.clone(),
                name: "elem_drop".to_string(),
            },
            func: &ELEM_DROP_RT_FUNC,
        },
    ]
}

#[derive(Clone)]
pub(crate) struct DependencyName {
    pub(crate) module: String,
    pub(crate) name: String,
}

#[derive(Default)]
pub(crate) struct DependencyStore<'a> {
    pub(crate) functions: Vec<FunctionDependency<'a>>,
    pub(crate) globals: Vec<RTGlobalImport>,
    pub(crate) tables: Vec<RTTableImport>,
    pub(crate) memories: Vec<RTMemoryImport>,
}

pub(crate) struct FunctionDependency<'a> {
    pub(crate) name: DependencyName,
    pub(crate) func: &'a Function,
}

impl DependencyName {
    pub(crate) fn symbol_name(&self) -> String {
        format!("{}.{}", self.module, self.name)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct RTMemoryImport {
    pub(crate) name: String,
    pub(crate) limits: Limits,
}

#[derive(Clone)]
pub(crate) struct RTGlobalImport {
    pub(crate) addr: *mut ValueRaw,
    pub(crate) r#type: GlobalType,
    pub(crate) idx: GlobalIdx,
}

#[derive(Clone)]
pub(crate) struct RTTableImport {
    pub(crate) name: String,
    pub(crate) instance_ref: *mut TableObject,
    pub(crate) r#type: TableType,
}

#[derive(Default, Clone)]
pub(crate) struct HostFunctionStorage {
    host_functions: HashMap<String, HashMap<String, Function>>,
}

impl HostFunctionStorage {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn insert<F, Params, Returns>(
        &mut self,
        module_name: &str,
        function_name: &str,
        callable: F,
    ) where
        F: IntoFunc<Params, Returns>,
    {
        self.host_functions
            .entry(module_name.to_string())
            .or_default()
            .insert(function_name.to_string(), callable.into_func());
    }

    pub(crate) fn get(&self, module_name: &str, function_name: &str) -> Option<&Function> {
        self.host_functions
            .get(module_name)
            .and_then(|module| module.get(function_name))
    }
}
