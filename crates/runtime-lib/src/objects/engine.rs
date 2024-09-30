use crate::objects::functions::BoundaryCCFuncTy;
use module::objects::{module::Module as WasmModule, value::ValueRaw};
use runtime_interface::RawPointer;
use std::{
    ops::{Deref, DerefMut},
    rc::Rc,
};
use wasm_types::{FuncIdx, GlobalIdx};

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[cfg(feature = "llvm")]
    #[error("LLVM execution error: {0}")]
    LLVMExecutionError(#[from] llvm_gen::ExecutionError),

    #[cfg(feature = "interp")]
    #[error("Interpreter error: {0}")]
    InterpreterError(#[from] interpreter::InterpreterError),

    #[error("Engine uninitialized. Call init function first.")]
    EngineUninitialized,

    #[error("Function with index {0} not exported.")]
    FunctionNotFound(FuncIdx),

    #[error("Module error: {0}")]
    ModuleError(#[from] module::ModuleError),
}

#[allow(private_interfaces)]
pub struct Engine(Box<dyn WasmEngine>);

pub trait WasmEngine {
    fn init(&mut self, wasm_module: WasmModule) -> Result<Rc<WasmModule>, EngineError>;

    fn set_symbol_addr(&mut self, name: &str, address: RawPointer);

    /// Get a raw function pointer that follows the engine backend's internal calling convention
    fn get_internal_function_ptr(&self, function_idx: FuncIdx) -> Result<RawPointer, EngineError>;

    /// Get a typed function pointer that follows the Boundary calling convention
    fn get_external_function_ptr(
        &self,
        function_idx: FuncIdx,
    ) -> Result<BoundaryCCFuncTy, EngineError>;

    fn get_global_value(&self, global_idx: GlobalIdx) -> Result<ValueRaw, EngineError>;
    fn set_global_addr(&mut self, global_idx: GlobalIdx, addr: RawPointer);
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
    use super::*;
    use wasm_types::GlobalIdx;

    pub(crate) struct LLVMEngine {
        context: Rc<llvm_gen::Context>,
        executor: llvm_gen::JITExecutor,
        wasm_module: Option<Rc<WasmModule>>,
    }

    impl LLVMEngine {
        pub(crate) fn new() -> Result<Self, EngineError> {
            let context = Rc::new(llvm_gen::Context::create());
            let executor = llvm_gen::JITExecutor::new(context.clone())?;
            Ok(Self {
                context,
                executor,
                wasm_module: None,
            })
        }
    }

    impl WasmEngine for LLVMEngine {
        fn init(&mut self, mut wasm_module: WasmModule) -> Result<Rc<WasmModule>, EngineError> {
            wasm_module.load_meta(llvm_gen::ModuleMetaLoader)?;
            wasm_module.load_all_functions(llvm_gen::FunctionLoader)?;

            let wasm_module = Rc::new(wasm_module);
            self.wasm_module = Some(wasm_module.clone());

            // Option 1: AOT llvm object buffer from cwasm available
            // -> add object file to executor
            if let Some(first_function) = wasm_module.meta.functions.first() {
                if let Some(precomp_info) = first_function.get_precompiled_llvm() {
                    self.executor.add_object_file(unsafe {
                        &*std::ptr::slice_from_raw_parts(
                            precomp_info.offset as *const u8,
                            precomp_info.size,
                        )
                    })?;
                    return Ok(wasm_module);
                }
            }

            // Option 2 || !Option 2: Additional resources contain LLVM module
            // = cached translation
            // -> add module to executor
            let llvm_module = wasm_module
                .additional_resources
                .first()
                .unwrap()
                .downcast_ref::<llvm_gen::LLVMAdditionalResources>()
                .unwrap()
                .module
                .clone();
            self.executor.add_module(llvm_module)?;

            return Ok(wasm_module);
        }

        fn get_global_value(&self, global_idx: GlobalIdx) -> Result<ValueRaw, EngineError> {
            Ok(self.executor.get_global_value(global_idx)?)
        }

        fn set_symbol_addr(&mut self, name: &str, addr: RawPointer) {
            self.executor.set_symbol_addr(name, addr);
        }

        fn get_external_function_ptr(
            &self,
            function_idx: FuncIdx,
        ) -> Result<BoundaryCCFuncTy, EngineError> {
            let func_name = self
                .wasm_module
                .as_ref()
                .ok_or(EngineError::EngineUninitialized)?
                .meta
                .exports
                .find_function_name(function_idx)
                .ok_or(EngineError::FunctionNotFound(function_idx))?;
            Ok(unsafe { std::mem::transmute_copy(&self.executor.get_symbol_addr(func_name)?) })
        }

        fn get_internal_function_ptr(
            &self,
            function_idx: FuncIdx,
        ) -> Result<RawPointer, EngineError> {
            Ok(self.executor.get_symbol_addr(&function_idx.to_string())?)
        }

        fn set_global_addr(&mut self, global_idx: GlobalIdx, addr: RawPointer) {
            self.executor.set_global_addr(global_idx, addr);
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
        fn init(&mut self, mut wasm_module: WasmModule) -> Result<Rc<WasmModule>, EngineError> {
            wasm_module.load_all_functions(parser::FunctionLoader)?;
            let wasm_module = Rc::new(wasm_module);
            self.interpreter.set_module(wasm_module.clone());
            Ok(wasm_module)
        }

        fn set_symbol_addr(&mut self, name: &str, address: RawPointer) {
            self.interpreter.register_symbol(name, address)
        }

        fn get_external_function_ptr(
            &self,
            function_idx: FuncIdx,
        ) -> Result<BoundaryCCFuncTy, EngineError> {
            todo!("Interpreter engine external function pointers")
        }

        fn get_internal_function_ptr(
            &self,
            function_idx: FuncIdx,
        ) -> Result<RawPointer, EngineError> {
            todo!("Interpreter engine internal function pointers")
        }

        fn get_global_value(&self, global_idx: GlobalIdx) -> Result<ValueRaw, EngineError> {
            todo!("Interpreter engine global values")
        }

        fn set_global_addr(&mut self, global_idx: GlobalIdx, addr: RawPointer) {
            todo!("Interpreter engine set global address")
        }
    }
}
