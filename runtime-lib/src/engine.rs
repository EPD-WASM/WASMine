use ir::structs::{module::Module as WasmModule, value::Value};
use runtime_interface::RawFunctionPtr;
use std::{
    ops::{Deref, DerefMut},
    rc::Rc,
};
use wasm_types::{GlobalIdx, ResType};

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[cfg(feature = "llvm")]
    #[error("LLVM execution error: {0}")]
    LLVMExecutionError(#[from] llvm_gen::ExecutionError),

    #[cfg(feature = "llvm")]
    #[error("LLVM translation error: {0}")]
    LLVMTranslationError(#[from] llvm_gen::TranslationError),

    #[error("Engine uninitialized. Call init function first.")]
    EngineUninitialized,
}

#[allow(private_interfaces)]
pub struct Engine(Box<dyn WasmEngine>);

pub trait WasmEngine {
    fn init(&mut self, wasm_module: Rc<WasmModule>) -> Result<(), EngineError>;
    fn register_symbol(&mut self, name: &str, address: RawFunctionPtr);

    fn get_raw_function_ptr_by_name(
        &self,
        function_name: &str,
    ) -> Result<RawFunctionPtr, EngineError>;
    fn get_global_value(&self, global_idx: GlobalIdx) -> Result<u64, EngineError>;

    fn run(
        &mut self,
        func_name: &str,
        func_ret_type: ResType,
        parameters: Vec<Value>,
        exec_ctxt: *mut runtime_interface::ExecutionContext,
    ) -> Result<Vec<Value>, EngineError>;
}

impl Engine {
    #[cfg(feature = "llvm")]
    pub fn llvm() -> Result<Self, EngineError> {
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
    use runtime_interface::ExecutionContext;
    use wasm_types::GlobalIdx;

    use super::*;

    pub(crate) struct LLVMEngine {
        context: Rc<llvm_gen::Context>,
        translator: llvm_gen::Translator,
        executor: llvm_gen::Executor,
        module_already_translated: bool,
        wasm_module: Option<Rc<WasmModule>>,
    }

    impl LLVMEngine {
        pub(crate) fn new() -> Result<Self, EngineError> {
            let context = Rc::new(llvm_gen::Context::create());
            let translator = llvm_gen::Translator::new(context.clone())?;
            let executor = llvm_gen::Executor::new(context.clone())?;
            Ok(Self {
                context,
                translator,
                executor,
                module_already_translated: false,
                wasm_module: None,
            })
        }
    }

    impl WasmEngine for LLVMEngine {
        fn init(&mut self, wasm_module: Rc<WasmModule>) -> Result<(), EngineError> {
            let llvm_module = self.translator.translate_module(wasm_module)?;
            self.executor.add_module(llvm_module)?;
            self.module_already_translated = true;
            Ok(())
        }

        fn get_global_value(&self, global_idx: GlobalIdx) -> Result<u64, EngineError> {
            Ok(self.executor.get_global_value(global_idx)?)
        }

        fn register_symbol(&mut self, name: &str, address: RawFunctionPtr) {
            self.executor.register_symbol(name, address);
        }

        fn get_raw_function_ptr_by_name(
            &self,
            function_name: &str,
        ) -> Result<RawFunctionPtr, EngineError> {
            Ok(self.executor.get_raw_by_name(function_name)?)
        }

        fn run(
            &mut self,
            func_name: &str,
            func_ret_type: ResType,
            parameters: Vec<Value>,
            exec_ctxt: *mut ExecutionContext,
        ) -> Result<Vec<Value>, EngineError> {
            if !self.module_already_translated {
                return Err(EngineError::EngineUninitialized);
            }
            Ok(unsafe {
                self.executor
                    .run(func_name, func_ret_type, parameters, exec_ctxt)?
            })
        }
    }
}
