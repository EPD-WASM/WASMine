use crate::{
    engine::{Engine, EngineError},
    globals::GlobalStorage,
    linker::RTImport,
    memory::{MemoryInstance, MemoryStorage},
    tables::TableInstance,
    Cluster, RuntimeError,
};
use cee_scape::call_with_sigsetjmp;
use core::{ffi, slice};
use ir::structs::{export::ExportDesc, module::Module as WasmModule, value::Value};
use nix::errno::Errno;
use runtime_interface::{ExecutionContext, RawFunctionPtr};
use std::{collections::HashMap, rc::Rc};
use wasm_types::{FuncIdx, FuncType, GlobalIdx, ValType};

#[derive(thiserror::Error, Debug)]
pub enum InstantiationError {
    #[error("Module does not contain start function.")]
    NoStartFunction,
    #[error("Allocation Failure ({0})")]
    AllocationFailure(Errno),
    #[error("Encountered reference to memory with index != 0. A maximum of one memory is allowed per wasm module.")]
    MemoryIdxNotZero,
    #[error("Offset into data segment was of invalid type '{0:}'")]
    InvalidDataOffsetType(ValType),
    #[error("Supplied datasource too small.")]
    DataSourceOOB,
    #[error("Memory init data too large for memory.")]
    MemoryInitOOB,
}

pub struct InstanceHandle<'a> {
    pub(crate) cluster: &'a Cluster,

    module: Rc<WasmModule>,
    engine: &'a mut Engine,
    execution_context: &'a mut ExecutionContext,

    function_exports: HashMap<String, FuncIdx>,
    global_exports: HashMap<String, GlobalIdx>,

    globals: &'a mut GlobalStorage,
    tables: &'a mut [TableInstance],
    memories: &'a mut [MemoryInstance],
}

impl<'a> InstanceHandle<'a> {
    pub(crate) fn new(
        cluster: &'a Cluster,
        m: Rc<WasmModule>,
        engine: Engine,
        imports: Vec<RTImport>,
    ) -> Result<Self, InstantiationError> {
        let globals = GlobalStorage::init_on_cluster(cluster, &m.globals, &imports)?;
        let tables = Self::init_tables_on_cluster(cluster, &m.tables, &imports)?;
        let memories = MemoryStorage::init_on_cluster(cluster, &m.memories, &m.datas, &imports)?;
        let engine = cluster.alloc_engine(engine);
        let execution_context = cluster.alloc_execution_context(ExecutionContext {
            tables_ptr: tables.as_mut_ptr() as *mut ffi::c_void,
            tables_len: tables.len(),
            globals_ptr: &mut globals.inner as *mut runtime_interface::GlobalStorage,
            globals_len: 1,
            memories_ptr: memories.as_mut_ptr() as *mut runtime_interface::MemoryInstance,
            memories_len: memories.len(),
            wasm_module: m.clone(),
            trap_return: None,
            trap_msg: None,
            recursion_size: 0,
        });

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

    pub(crate) fn query_start_function(&self) -> Result<FuncIdx, InstantiationError> {
        if let Some(start_func_idx) = self.module.entry_point {
            return Ok(start_func_idx);
        }
        self.find_func("_start")
            .or_else(|_| self.find_func("run"))
            .or(Err(InstantiationError::NoStartFunction))
    }

    pub fn find_func(&self, name: &str) -> Result<FuncIdx, InstantiationError> {
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
        Err(InstantiationError::NoStartFunction)
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
        self.engine.get_raw_function_ptr(name)
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

    fn init_globals_from_runtime(&mut self) {
        for (idx, global) in self.globals.inner.globals.iter().enumerate() {
            if global.addr.is_null() {
                panic!("missing global initialization")
            }
            self.engine
                .register_symbol(&format!("global_{}", idx), global.addr as RawFunctionPtr);
        }
    }

    pub fn run_by_name(
        &mut self,
        func_name: &str,
        input_params: Vec<Value>,
    ) -> Result<Vec<Value>, RuntimeError> {
        self.init_globals_from_runtime();

        let func_idx = self.find_func(func_name)?;
        let function_type = self.get_function_type_from_func_idx(func_idx);
        let res_ty = function_type.1.clone();

        let mut res_opt = None;
        let jmp_res = call_with_sigsetjmp(true, |jmp_buf| {
            self.execution_context.trap_return = Some(jmp_buf);
            res_opt = Some(self.engine.run(
                func_name,
                res_ty,
                input_params,
                self.execution_context as *mut ExecutionContext,
            ));
            self.execution_context.trap_return = None;
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
                    self.execution_context as *const runtime_interface::ExecutionContext
                        as *mut runtime_interface::ExecutionContext,
                    1,
                )[0],

                globals: &mut slice::from_raw_parts_mut(
                    self.globals as *const GlobalStorage as *mut crate::globals::GlobalStorage,
                    1,
                )[0],
                tables: slice::from_raw_parts_mut(
                    self.tables.as_ptr() as *mut TableInstance,
                    self.tables.len(),
                ),
                memories: slice::from_raw_parts_mut(
                    self.memories.as_ptr() as *mut MemoryInstance,
                    self.memories.len(),
                ),
            }
        }
    }
}
