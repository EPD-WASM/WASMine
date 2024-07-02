use std::{
    ops::{Deref, DerefMut},
    rc::Rc,
};

use crate::error::RuntimeError;
use ir::structs::{module::Module as WasmModule, value::Value};
use runtime_interface::RawFunctionPtr;
use wasm_types::ResType;

#[allow(private_interfaces)]
pub struct Engine(Box<dyn WasmEngine>);

pub trait WasmEngine {
    fn set_wasm_module(&mut self, wasm_module: Rc<WasmModule>);
    fn register_symbol(&mut self, name: &str, address: RawFunctionPtr);

    fn get_raw_function_ptr(&self, function_name: &str) -> Result<RawFunctionPtr, RuntimeError>;
    fn run(
        &mut self,
        func_name: &str,
        func_ret_type: ResType,
        parameters: Vec<Value>,
        exec_ctxt: *mut runtime_interface::ExecutionContext,
    ) -> Result<Vec<Value>, RuntimeError>;
}

impl Engine {
    #[cfg(feature = "llvm")]
    pub fn llvm() -> Result<Self, RuntimeError> {
        Ok(Self(Box::new(llvm_engine_impl::LLVMEngine::new()?)))
    }
}

impl Deref for Engine {
    type Target = Box<dyn WasmEngine>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Engine {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(feature = "llvm")]
mod llvm_engine_impl {
    /* to add */
}
