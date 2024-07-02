use crate::{
    engine::Engine, error::RuntimeError, memory::MemoryStorage, runtime::Runtime,
    tables::TableInstance,
};
use ir::structs::{export::ExportDesc, module::Module as WasmModule, value::Value};
use runtime_interface::{ExecutionContext, GlobalInstance, RawFunctionPtr};
use std::{pin::Pin, rc::Rc};
use wasm_types::{FuncIdx, FuncType};

pub struct WasmModuleInstance {
    module: Rc<WasmModule>,
    runtime: Pin<Box<Runtime>>,
    imported_memories: Vec<runtime_interface::MemoryInstance>,
    ee: Engine,
    // keep one context for now => this should be handled by run()-caller later
    execution_context: Option<ExecutionContext>,
}

impl WasmModuleInstance {
    pub(crate) fn new(m: Rc<WasmModule>, mut engine: Engine) -> Self {
        engine.set_wasm_module(m.clone());
        Self {
            module: m.clone(),
            runtime: Box::pin(Runtime::new(m)),
            ee: engine,
            imported_memories: Vec::new(),
            execution_context: None,
        }
    }

    pub(crate) fn query_start_function(&self) -> Result<FuncIdx, RuntimeError> {
        if let Some(start_func_idx) = self.module.entry_point {
            return Ok(start_func_idx);
        }
        self.find_func("_start")
            .or_else(|_| self.find_func("run"))
            .or(Err(RuntimeError::NoStartFunction))
    }

    pub fn find_func(&self, name: &str) -> Result<FuncIdx, RuntimeError> {
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
        Err(RuntimeError::NoStartFunction)
    }

    pub fn get_function_type(&self, func_idx: u32) -> &FuncType {
        &self.module.function_types[self.module.ir.functions[func_idx as usize].type_idx as usize]
    }

    pub(crate) fn add_func_import(&mut self, name: &str, callable: RawFunctionPtr) {
        self.ee.register_symbol(name, callable)
    }

    pub(crate) fn add_memory_import(&mut self, instance: runtime_interface::MemoryInstance) {
        self.imported_memories.push(instance)
    }

    pub(crate) fn add_table_import(&mut self, instance: TableInstance) {
        self.runtime.tables.push(instance)
    }

    pub(crate) fn add_global_import(&mut self, instance: GlobalInstance) {
        self.runtime.globals.import(instance);
    }

    pub fn get_raw_function_ptr(&self, name: &str) -> Result<RawFunctionPtr, RuntimeError> {
        self.ee.get_raw_function_ptr(name)
    }

    pub fn wasm_module(&self) -> &Rc<WasmModule> {
        &self.module
    }

    pub fn runtime(&self) -> &Runtime {
        &self.runtime
    }

    fn init_globals_from_runtime(&mut self) {
        for (idx, global) in self.runtime.globals.inner.globals.iter().enumerate() {
            if global.addr.is_null() {
                panic!("missing global initialization")
            }
            self.ee
                .register_symbol(&format!("global_{}", idx), global.addr as RawFunctionPtr);
        }
    }

    fn create_execution_context(&self) -> Result<ExecutionContext, RuntimeError> {
        self.runtime
            .create_execution_context(&self.imported_memories)
    }

    pub fn copy_execution_ctxt_memory(
        &mut self,
    ) -> Result<runtime_interface::MemoryInstance, RuntimeError> {
        if self.execution_context.is_none() {
            self.execution_context = Some(self.create_execution_context()?);
        }

        if let Some(exec_ctxt) = &self.execution_context {
            let mem_storage = MemoryStorage::from_raw_parts(
                exec_ctxt.memories_ptr,
                exec_ctxt.memories_len,
                exec_ctxt.memories_cap,
            );
            Ok(mem_storage[0].0.clone())
        } else {
            unreachable!()
        }
    }

    pub fn run_by_name(
        &mut self,
        func_name: &str,
        input_params: Vec<Value>,
    ) -> Result<Vec<Value>, RuntimeError> {
        self.init_globals_from_runtime();
        if self.execution_context.is_none() {
            self.execution_context = Some(self.create_execution_context()?);
        }

        let func_idx = self.find_func(func_name)?;
        let function_type = self.get_function_type(func_idx);
        let res_ty = function_type.1.clone();
        self.ee.run(
            func_name,
            res_ty,
            input_params,
            self.execution_context.as_mut().unwrap() as *mut ExecutionContext,
        )
    }
}
