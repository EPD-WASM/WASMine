use crate::{
    engine::{Engine, EngineError},
    execution_context::ExecutionContextWrapper,
    globals::GlobalStorage,
    linker::{rt_func_imports, RTImportCollection},
    memory::{MemoryError, MemoryInstance, MemoryStorage},
    signals::SignalHandler,
    tables::{TableError, TableInstance},
    Cluster, RuntimeError,
};
use cee_scape::call_with_sigsetjmp;
use core::{ffi, slice};
use ir::structs::{export::ExportDesc, module::Module as WasmModule, value::Value};
use runtime_interface::{ExecutionContext, GlobalInstance, RawFunctionPtr};
use std::{collections::HashMap, ptr::null_mut, rc::Rc};
use wasm_types::{FuncIdx, FuncType, GlobalIdx, MemIdx, TableIdx};

#[derive(thiserror::Error, Debug)]
pub enum InstantiationError {
    #[error("Module does not contain start function.")]
    NoStartFunction,
    #[error("Error during table initialization: {0}")]
    TableError(#[from] TableError),
    #[error("Error during memory initialization: {0}")]
    MemoryError(#[from] MemoryError),
    #[error("Function not found: {0}")]
    FunctionNotFound(String),
}

pub struct InstanceHandle<'a> {
    pub(crate) cluster: &'a Cluster,

    module: Rc<WasmModule>,
    engine: &'a mut Engine,
    execution_context: &'a mut ExecutionContext,

    function_exports: HashMap<String, FuncIdx>,
    global_exports: HashMap<String, GlobalIdx>,

    globals: &'a mut GlobalStorage,
    tables: Vec<TableInstance<'a>>,
    memories: &'a mut [MemoryInstance],
}

impl<'a> InstanceHandle<'a> {
    pub(crate) fn new(
        cluster: &'a Cluster,
        m: Rc<WasmModule>,
        engine: Engine,
        imports: RTImportCollection,
    ) -> Result<Self, InstantiationError> {
        let engine = cluster.alloc_engine(engine);
        let execution_context = cluster.alloc_execution_context(ExecutionContext {
            tables_ptr: null_mut(),
            tables_len: 0,
            globals_ptr: null_mut(),
            globals_len: 0,
            memories_ptr: null_mut(),
            memories_len: 0,
            wasm_module: m.clone(),
            engine: engine as *mut Engine as *mut ffi::c_void,
            trap_msg: None,
            recursion_size: 0,
            id: 0,
        });

        for f in imports.functions.iter() {
            engine.register_symbol(&format!("{}_imported", f.name), f.callable);
            let rt_ptr = f.execution_context.unwrap_or(execution_context);
            engine.register_symbol(
                &format!("import_{}_rt_ptr", f.name.split_once('.').unwrap().0),
                rt_ptr as RawFunctionPtr,
            );
        }

        for import in rt_func_imports() {
            engine.register_symbol(import.0, import.1.callable)
        }

        // initialize globals
        let globals =
            GlobalStorage::init_on_cluster(cluster, &m.globals, &imports.globals, engine)?;
        execution_context.globals_ptr = &mut globals.inner as *mut runtime_interface::GlobalStorage;
        execution_context.globals_len = 1;

        // initialize tables
        let tables = Self::init_tables_on_cluster(
            m.as_ref(),
            engine,
            cluster,
            &m.tables,
            &m.elements,
            &imports.tables,
            &globals.inner,
        )?;
        execution_context.tables_ptr = tables.as_ptr() as *mut ffi::c_void;
        execution_context.tables_len = tables.len();

        // initialize memories
        let memories = MemoryStorage::init_on_cluster(
            cluster,
            &m.memories,
            &m.datas,
            &imports.memories,
            &globals.inner,
        )?;
        execution_context.memories_ptr =
            memories.as_mut_ptr() as *mut runtime_interface::MemoryInstance;
        execution_context.memories_len = memories.len();

        SignalHandler::register_globally();
        Ok(Self {
            function_exports: Self::collect_function_exports(&m),
            global_exports: Self::collect_global_exports(&m),
            module: m.clone(),
            engine,
            execution_context,
            globals,
            tables,
            memories,
            cluster,
        })
    }

    fn collect_function_exports(m: &Rc<WasmModule>) -> HashMap<String, FuncIdx> {
        m.exports
            .iter()
            .filter_map(|e| match e.desc {
                ExportDesc::Func(idx) => Some((e.name.clone(), idx)),
                _ => None,
            })
            .collect()
    }

    fn collect_global_exports(m: &Rc<WasmModule>) -> HashMap<String, GlobalIdx> {
        m.exports
            .iter()
            .filter_map(|e| match e.desc {
                ExportDesc::Global(idx) => Some((e.name.clone(), idx)),
                _ => None,
            })
            .collect()
    }

    pub fn query_start_function(&self) -> Result<FuncIdx, InstantiationError> {
        if let Some(start_func_idx) = self.module.entry_point {
            return Ok(start_func_idx);
        }
        self.find_exported_func_idx("_start")
            .or_else(|_| self.find_exported_func_idx("run"))
            .or(Err(InstantiationError::NoStartFunction))
    }

    pub fn find_exported_func_idx(&self, name: &str) -> Result<FuncIdx, InstantiationError> {
        if let Some(entry) = self.module.entry_point {
            if name == format!("func_{}", entry) {
                return Ok(entry);
            }
        }
        if let Some(f) = self
            .module
            .exports
            .iter()
            .filter(|e| matches!(e.desc, ExportDesc::Func(_)))
            .find(|e| e.name == name)
        {
            return match f.desc {
                ExportDesc::Func(idx) => Ok(idx),
                _ => unreachable!(),
            };
        }
        Err(InstantiationError::FunctionNotFound(name.to_string()))
    }

    pub fn get_function_type_from_func_idx(&self, func_idx: u32) -> &FuncType {
        &self.module.function_types[self.module.ir.functions[func_idx as usize].type_idx as usize]
    }

    pub fn get_function_type_from_name(&self, name: &str) -> Option<&FuncType> {
        self.function_exports.get(name).map(|&idx| {
            &self.module.function_types[self.module.ir.functions[idx as usize].type_idx as usize]
        })
    }

    pub fn get_raw_function_ptr(&self, name: &str) -> Result<RawFunctionPtr, EngineError> {
        self.engine.get_raw_function_ptr_by_name(name)
    }

    pub fn wasm_module(&self) -> &Rc<WasmModule> {
        &self.module
    }

    pub fn extract_global_value_by_idx(&self, idx: usize) -> Value {
        let global_addr = self.globals.inner.globals[idx].addr;
        let global_valty = match &self.module.globals[idx].r#type {
            wasm_types::GlobalType::Const(ty) => ty,
            wasm_types::GlobalType::Mut(ty) => ty,
        };
        let global_val = unsafe { *(global_addr) };
        Value::from_generic(*global_valty, global_val)
    }

    pub fn extract_global_value_by_name(&self, name: &str) -> Option<Value> {
        self.global_exports
            .get(name)
            .map(|i| self.extract_global_value_by_idx(*i as usize))
    }

    // fn init_globals_from_runtime(&mut self) {
    //     for (idx, global) in self.globals.inner.globals.iter().enumerate() {
    //         if global.addr.is_null() {
    //             panic!("missing global initialization")
    //         }
    //         self.engine
    //             .register_symbol(&format!("global_{}", idx), global.addr as RawFunctionPtr);
    //     }
    // }

    pub(crate) fn memories(&self, mem_idx: MemIdx) -> &MemoryInstance {
        &self.memories[mem_idx as usize]
    }

    pub(crate) fn globals(&self, global_idx: GlobalIdx) -> &GlobalInstance {
        &self.globals.inner.globals[global_idx as usize]
    }

    pub(crate) fn tables(&self, table_idx: TableIdx) -> &TableInstance {
        &self.tables[table_idx as usize]
    }

    pub(crate) fn execution_context(&self) -> *mut ExecutionContext {
        self.execution_context as *const _ as *mut _
    }

    pub fn run_by_name(
        &mut self,
        func_name: &str,
        input_params: Vec<Value>,
    ) -> Result<Vec<Value>, RuntimeError> {
        let func_idx = self.find_exported_func_idx(func_name)?;
        let function_type = self.get_function_type_from_func_idx(func_idx);
        let res_ty = function_type.1.clone();

        let mut res_opt = None;
        let jmp_res = call_with_sigsetjmp(true, |jmp_buf| {
            ExecutionContextWrapper::set_trap_return_point(jmp_buf);
            SignalHandler::set_thread_executing_wasm();

            res_opt = Some(self.engine.run_by_idx(
                func_idx,
                res_ty,
                input_params,
                self.execution_context as *mut ExecutionContext,
            ));
            SignalHandler::unset_thread_executing_wasm();
            0
        });
        if jmp_res != 0 {
            return Err(RuntimeError::Trap(
                self.execution_context
                    .trap_msg
                    .clone()
                    .unwrap_or("<no msg>".to_string()),
            ));
        }
        Ok(res_opt.unwrap()?)
    }
}

impl<'a> Clone for InstanceHandle<'a> {
    // even though this looks bad, it is merely here to fool the borrow checker again. All the contained references are unsafe and non-owning anyways.
    // Sole owner of all referenced objects is the cluster object with lifetime 'a.
    fn clone(&self) -> Self {
        SignalHandler::register_globally();
        unsafe {
            Self {
                module: self.module.clone(),
                function_exports: self.function_exports.clone(),
                global_exports: self.global_exports.clone(),

                cluster: &slice::from_raw_parts(self.cluster as *const Cluster, 1)[0],

                engine: &mut slice::from_raw_parts_mut(
                    self.engine as *const Engine as *mut Engine,
                    1,
                )[0],
                execution_context: &mut slice::from_raw_parts_mut(
                    self.execution_context as *const _ as *mut _,
                    1,
                )[0],

                globals: &mut slice::from_raw_parts_mut(self.globals as *const _ as *mut _, 1)[0],
                tables: self
                    .tables
                    .iter()
                    .map(|t| TableInstance {
                        values: &mut slice::from_raw_parts_mut(t.values as *const _ as *mut _, 1)
                            [0],
                        ty: t.ty,
                    })
                    .collect(),
                memories: slice::from_raw_parts_mut(
                    self.memories.as_ptr() as *mut _,
                    self.memories.len(),
                ),
            }
        }
    }
}

impl Drop for InstanceHandle<'_> {
    fn drop(&mut self) {
        SignalHandler::deregister_globally();
    }
}
