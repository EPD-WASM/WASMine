use crate::{
    engine::{Engine, EngineError},
    module_instance::{InstanceHandle, InstantiationError},
    segmented_list::SegmentedList,
    wasi, Cluster,
};
use ir::structs::{
    data::Data, element::Element, export::ExportDesc, module::Module as WasmModule, value::Value,
};
use runtime_interface::RawFunctionPtr;
use std::{collections::HashMap, rc::Rc};
use wasm_types::{FuncType, GlobalType, ImportDesc, Limits, TableType};

#[derive(thiserror::Error, Debug)]
pub enum LinkingError {
    #[error("Module cluster mismatch. Bound linker received module from foreign cluster.")]
    ClusterMismatch,
    #[error("Missing required module '{module_name}'.")]
    ModuleNotFound { module_name: String },
    #[error("Function type mismatch. Requested: {requested:?}, Actual: {actual:?}.")]
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
    #[error("Global '{global_name}' not found in module '{module_name}'.")]
    GlobalNotFound {
        global_name: String,
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
    ) -> Result<Vec<RTImport>, LinkingError> {
        module
            .imports
            .iter()
            .map(|import| {
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
                        let requested_function_type =
                            module.function_types[*type_idx as usize].clone();
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
                        match exporting_module.get_raw_function_ptr(&import.name) {
                            Ok(callable) => Ok(RTImport::Func(RTFuncImport {
                                name: import.name.clone(),
                                function_type: requested_function_type,
                                callable,
                            })),
                            Err(e) => Err(e.into()),
                        }
                    }
                    ImportDesc::Global(requested_type) => {
                        let exported_global = match exporting_module
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
                            &exporting_module.wasm_module().globals[*exported_global as usize];
                        let actual_type = &exported_global.r#type;
                        if requested_type != actual_type {
                            return Err(LinkingError::GlobalTypeMismatch {
                                requested: *requested_type,
                                actual: *actual_type,
                            });
                        }
                        Ok(RTImport::Global(RTGlobalImport {
                            name: import.name.clone(),
                            init: exported_global.init.clone(),
                            r#type: *requested_type,
                        }))
                    }
                    ImportDesc::Mem(limits) => {
                        todo!()
                    }
                    ImportDesc::Table(table_type) => {
                        todo!()
                    }
                }
            })
            .collect()
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

#[derive(Clone, Debug)]
pub(crate) struct RTFuncImport {
    pub(crate) name: String,
    pub(crate) function_type: FuncType,
    pub(crate) callable: RawFunctionPtr,
}

#[derive(Clone, Debug)]
pub(crate) struct RTMemoryImport {
    pub(crate) name: String,
    pub(crate) datas: Vec<Data>,
    pub(crate) limits: Limits,
}

#[derive(Clone)]
pub(crate) struct RTGlobalImport {
    pub(crate) name: String,
    pub(crate) init: Value,
    pub(crate) r#type: GlobalType,
}

#[derive(Clone)]
pub(crate) struct RTTableImport {
    pub(crate) name: String,
    pub(crate) r#type: TableType,
    pub(crate) elements: Vec<Element>,
}

pub(crate) enum RTImport {
    Func(RTFuncImport),
    Memory(RTMemoryImport),
    Global(RTGlobalImport),
    Table(RTTableImport),
}
