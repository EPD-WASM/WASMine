use crate::{
    abstraction::{context::Context, execution_engine::ExecutionEngine, module::Module},
    error::ExecutionError,
};
use ir::structs::value::ValueRaw;
use runtime_interface::RawFunctionPtr;
use std::rc::Rc;
use wasm_types::GlobalIdx;

pub struct Executor {
    execution_engine: ExecutionEngine,
    #[allow(dead_code)] // hold on to the context to prevent it from being dropped
    context: Rc<Context>,
}

impl Executor {
    pub fn new(context: Rc<Context>) -> Result<Self, ExecutionError> {
        Ok(Self {
            execution_engine: ExecutionEngine::init()?,
            context,
        })
    }

    pub fn get_raw_by_name(&self, function_name: &str) -> Result<RawFunctionPtr, ExecutionError> {
        self.execution_engine
            .find_func_address_by_name(function_name)
    }

    pub fn register_symbol(&mut self, name: &str, address: *const core::ffi::c_void) {
        self.execution_engine.register_symbol(name, address);
    }

    pub fn add_module(&mut self, module: Rc<Module>) -> Result<(), ExecutionError> {
        self.execution_engine.optimize_module(&module)?;

        #[cfg(debug_assertions)]
        module.print_to_file();

        self.execution_engine.add_llvm_module(module)
    }

    pub fn get_global_value(&self, global_idx: GlobalIdx) -> Result<ValueRaw, ExecutionError> {
        self.execution_engine
            .get_global_value(&format!("__wasmine_global__{}", global_idx))
    }
}
