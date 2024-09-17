use crate::{
    abstraction::{
        context::Context, lljit::JITExecutionEngine, module::Module, pass_manager::PassManager,
    },
    error::ExecutionError,
};
use module::objects::value::ValueRaw;
use runtime_interface::RawPointer;
use std::rc::Rc;
use wasm_types::GlobalIdx;

pub struct JITExecutor {
    execution_engine: JITExecutionEngine,
    #[allow(dead_code)] // hold on to the context to prevent it from being dropped
    context: Rc<Context>,
}

impl JITExecutor {
    pub fn new(context: Rc<Context>) -> Result<Self, ExecutionError> {
        Ok(Self {
            execution_engine: JITExecutionEngine::init()?,
            context,
        })
    }

    pub fn get_symbol_addr(&self, name: &str) -> Result<RawPointer, ExecutionError> {
        self.execution_engine.get_symbol_addr(name)
    }

    pub fn set_symbol_addr(&mut self, name: &str, address: RawPointer) {
        self.execution_engine.register_symbol(name, address);
    }

    pub fn add_module(&mut self, module: Rc<Module>) -> Result<(), ExecutionError> {
        PassManager::optimize_module(&module)?;

        #[cfg(debug_assertions)]
        module.print_to_file();

        self.execution_engine.add_llvm_module(module)
    }

    pub fn get_module_as_object_buffer(&self, module_idx: usize) -> Result<&[u8], ExecutionError> {
        self.execution_engine
            .get_module_as_object_buffer(module_idx)
    }

    /// Add object file to the JIT compiler
    ///
    /// # Warning
    /// Does not take ownership of the object file buffer. Buffer must be kept alive until the JIT compiler is dropped.
    pub fn add_object_file(&mut self, obj_file: &[u8]) -> Result<(), ExecutionError> {
        self.execution_engine.add_object_file(obj_file)
    }

    pub fn get_global_value(&self, global_idx: GlobalIdx) -> Result<ValueRaw, ExecutionError> {
        self.execution_engine
            .get_global(&format!("__wasmine_global__{global_idx}"))
    }

    pub fn set_global_addr(&mut self, global_idx: GlobalIdx, addr: RawPointer) {
        self.execution_engine
            .register_symbol(&format!("__wasmine_global__{global_idx}"), addr)
    }
}
