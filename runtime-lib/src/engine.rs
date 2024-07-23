use ir::structs::{
    module::Module as WasmModule,
    value::{Value, ValueRaw},
};
use runtime_interface::RawFunctionPtr;
use std::{
    ops::{Deref, DerefMut},
    rc::Rc,
};
use wasm_types::{FuncIdx, GlobalIdx, ResType};

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[cfg(feature = "llvm")]
    #[error("LLVM execution error: {0}")]
    LLVMExecutionError(#[from] llvm_gen::ExecutionError),

    #[cfg(feature = "llvm")]
    #[error("LLVM translation error: {0}")]
    LLVMTranslationError(#[from] llvm_gen::TranslationError),

    #[cfg(feature = "interp")]
    #[error("Interpreter error: {0}")]
    InterpreterError(#[from] interpreter::InterpreterError),

    #[error("Engine uninitialized. Call init function first.")]
    EngineUninitialized,

    #[error("Function with index {0} not exported.")]
    FunctionNotFound(FuncIdx),
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
    fn get_global_value(&self, global_idx: GlobalIdx) -> Result<ValueRaw, EngineError>;

    fn run_by_idx(
        &mut self,
        func_idx: u32,
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
    #[cfg(feature = "interp")]
    pub fn interpreter() -> Result<Self, EngineError> {
        Ok(Self(Box::new(
            interpreter_engine_impl::InterpreterEngine::new()?,
        )))
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
    use ir::function::Function;
    use runtime_interface::ExecutionContext;
    use wasm_types::{FuncIdx, GlobalIdx};

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
            self.wasm_module = Some(wasm_module.clone());
            let llvm_module = self.translator.translate_module(wasm_module)?;
            self.executor.add_module(llvm_module)?;
            self.module_already_translated = true;
            Ok(())
        }

        fn get_global_value(&self, global_idx: GlobalIdx) -> Result<ValueRaw, EngineError> {
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

        fn run_by_idx(
            &mut self,
            func_idx: FuncIdx,
            func_ret_type: ResType,
            parameters: Vec<Value>,
            exec_ctxt: *mut ExecutionContext,
        ) -> Result<Vec<Value>, EngineError> {
            if !self.module_already_translated {
                return Err(EngineError::EngineUninitialized);
            }
            Ok(unsafe {
                let func_name = match Function::query_function_name(
                    func_idx,
                    self.wasm_module
                        .as_ref()
                        .expect("already checked initialization"),
                ) {
                    Some(name) => name,
                    None => return Err(EngineError::FunctionNotFound(func_idx)),
                };
                self.executor
                    .run(func_name.as_str(), func_ret_type, parameters, exec_ctxt)?
            })
        }
    }
}

#[cfg(feature = "interp")]
mod interpreter_engine_impl {
    use super::*;
    use interpreter::Interpreter;

    pub(crate) struct InterpreterEngine {
        interpreter: Interpreter,
    }

    impl InterpreterEngine {
        pub(crate) fn new() -> Result<Self, EngineError> {
            Ok(Self {
                interpreter: Interpreter::new(),
            })
        }
    }

    impl WasmEngine for InterpreterEngine {
        // this is to set the module to be run so it does not have to be provided when the Engine is created
        fn init(&mut self, wasm_module: Rc<WasmModule>) -> Result<(), EngineError> {
            self.interpreter.set_module(wasm_module);
            Ok(())
        }

        fn register_symbol(&mut self, name: &str, address: RawFunctionPtr) {
            self.interpreter.register_symbol(name, address)
        }

        fn run_by_idx(
            &mut self,
            func_idx: u32,
            func_ret_type: ResType,
            parameters: Vec<Value>,
            exec_ctx: *mut runtime_interface::ExecutionContext,
        ) -> Result<Vec<Value>, EngineError> {
            unsafe {
                self.interpreter
                    .run(func_idx, parameters, exec_ctx)
                    .map_err(|e| e.into())
            }
        }

        fn get_raw_function_ptr_by_name(
            &self,
            function_name: &str,
        ) -> Result<RawFunctionPtr, EngineError> {
            todo!("Interpreter engine raw pointers")
        }

        fn get_global_value(&self, global_idx: GlobalIdx) -> Result<ValueRaw, EngineError> {
            todo!("Interpreter engine global values")
        }
    }
}
