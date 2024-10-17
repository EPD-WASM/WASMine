use crate::{objects::execution_context::TrapUnwrap, RuntimeError};
use interpreter::{Interpreter, InterpreterError};
use module::objects::{module::Module as WasmModule, value::ValueRaw};
use runtime_interface::RawPointer;
use std::{
    cell::RefCell,
    ops::{Deref, DerefMut},
    rc::Rc,
};
use wasm_types::{FuncIdx, FuncType, GlobalIdx};

use super::functions::{BoundaryCCFuncTy, Function};

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

    #[cfg(feature = "llvm")]
    #[error("LLVM translation error: {0}")]
    TranslationError(#[from] llvm_gen::TranslationError),
}

#[allow(private_interfaces)]
pub struct Engine(Box<dyn WasmEngine>);

pub type InterpreterInfo = (Rc<RefCell<Interpreter>>, FuncIdx, FuncType);

// I tried it with just tuples but it broke me
/// Boundary calling convention function pointer and a little extra spice because the interpreter needs it like I need sleep
pub struct BoundaryFuncAndCtx {
    pub func: BoundaryCCFuncTy,
    pub info: Option<InterpreterInfo>,
}

pub trait WasmEngine {
    fn init(&mut self, wasm_module: Rc<WasmModule>) -> Result<(), EngineError>;

    fn set_symbol_addr(&mut self, name: &str, address: RawPointer);

    /// Get a raw function pointer that follows the engine backend's internal calling convention
    fn get_internal_function_ptr(&self, function_idx: FuncIdx) -> Result<RawPointer, EngineError>;

    /// Get a typed function pointer that follows the Boundary calling convention
    fn get_external_function_ptr(
        &self,
        function_idx: FuncIdx,
    ) -> Result<BoundaryFuncAndCtx, EngineError>;

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
        executor: Option<llvm_gen::JITExecutor>,
        wasm_module: Option<Rc<WasmModule>>,
    }

    impl LLVMEngine {
        pub(crate) fn new() -> Result<Self, EngineError> {
            Ok(Self {
                executor: None,
                wasm_module: None,
            })
        }
    }

    impl WasmEngine for LLVMEngine {
        fn init(&mut self, wasm_module: Rc<WasmModule>) -> Result<(), EngineError> {
            self.wasm_module = Some(wasm_module.clone());
            self.executor = Some(llvm_gen::JITExecutor::new(wasm_module)?);
            Ok(())
        }

        fn get_global_value(&self, global_idx: GlobalIdx) -> Result<ValueRaw, EngineError> {
            Ok(self
                .executor
                .as_ref()
                .unwrap()
                .get_global_value(global_idx)?)
        }

        fn set_symbol_addr(&mut self, name: &str, addr: RawPointer) {
            self.executor.as_mut().unwrap().set_symbol_addr(name, addr);
        }

        fn get_external_function_ptr(
            &self,
            function_idx: FuncIdx,
        ) -> Result<BoundaryFuncAndCtx, EngineError> {
            let func_name = self
                .wasm_module
                .as_ref()
                .ok_or(EngineError::EngineUninitialized)?
                .meta
                .exports
                .find_function_name(function_idx)
                .ok_or(EngineError::FunctionNotFound(function_idx))?;
            let boundary_cc_func = unsafe {
                std::mem::transmute_copy(
                    &self.executor.as_ref().unwrap().get_symbol_addr(func_name)?,
                )
            };

            Ok(BoundaryFuncAndCtx {
                func: boundary_cc_func,
                info: None,
            })
        }

        fn get_internal_function_ptr(
            &self,
            function_idx: FuncIdx,
        ) -> Result<RawPointer, EngineError> {
            Ok(self
                .executor
                .as_ref()
                .unwrap()
                .get_symbol_addr(&function_idx.to_string())?)
        }

        fn set_global_addr(&mut self, global_idx: GlobalIdx, addr: RawPointer) {
            self.executor
                .as_mut()
                .unwrap()
                .set_global_addr(global_idx, addr);
        }
    }
}

#[cfg(feature = "interp")]
mod interpreter_engine_impl {
    use core::{panic, slice};
    use std::{cell::RefCell, mem::transmute, ptr::NonNull};

    use crate::objects::functions::{CalleeCtxt, HostFuncRawContainer};

    use super::*;
    use interpreter::Interpreter;

    use module::{instructions::FunctionIR, objects::value::Value};
    use runtime_interface::ExecutionContext;
    use wasm_types::FuncType;

    pub(crate) struct InterpreterEngine {
        interpreter: Rc<RefCell<Interpreter>>,
        module: Option<Rc<WasmModule>>,
    }

    impl InterpreterEngine {
        pub(crate) fn new() -> Result<Self, EngineError> {
            Ok(Self {
                interpreter: Rc::new(RefCell::new(Interpreter::new())),
                module: None,
            })
        }
    }

    impl WasmEngine for InterpreterEngine {
        // this is to set the module to be run so it does not have to be provided when the Engine is created
        fn init(&mut self, wasm_module: Rc<WasmModule>) -> Result<(), EngineError> {
            wasm_module.load_all_functions(parser::FunctionLoader)?;
            self.interpreter
                .borrow_mut()
                .set_module(wasm_module.clone());
            self.module = Some(wasm_module.clone());

            let artifact_lock = self
                .module
                .as_ref()
                .unwrap()
                .artifact_registry
                .read()
                .unwrap();
            let ir_locked = match artifact_lock.get("ir") {
                Some(ir) => ir,
                None => return Err(EngineError::InterpreterError(InterpreterError::NoIR)),
            };
            let ir_read = ir_locked.read().unwrap();
            let ir: &Vec<FunctionIR> = ir_read.downcast_ref().unwrap();

            self.interpreter.borrow_mut().set_ir(Rc::new(ir.clone()));

            Ok(())
        }

        fn set_symbol_addr(&mut self, name: &str, address: RawPointer) {
            log::debug!(
                "Interpreter engine setting symbol address for {} to {:?}",
                name,
                address
            );
            self.interpreter.borrow_mut().set_symbol_addr(name, address)
        }

        fn get_external_function_ptr(
            &self,
            function_idx: FuncIdx,
        ) -> Result<BoundaryFuncAndCtx, EngineError> {
            log::debug!("Get external function ptr for func idx: {}", function_idx);

            let module = self
                .module
                .as_ref()
                .ok_or(EngineError::EngineUninitialized)?;

            let func_name = module
                .meta
                .exports
                .find_function_name(function_idx)
                .ok_or(EngineError::FunctionNotFound(function_idx))?;

            log::debug!("Creating function wrapper for {func_name}");

            // panicking in here (and anything called by this) causes a SIGABRT (I think)
            unsafe extern "C" fn func(
                ctx_raw: CalleeCtxt,
                args_raw: *const ValueRaw,
                rets_raw: *mut ValueRaw,
            ) {
                log::trace!("Wrapper function called");
                if ctx_raw.host_func_context.is_null() {
                    panic!("host_func_context is null");
                }
                let host_func_raw_container = unsafe { &*ctx_raw.host_func_context };

                let ctx_any_ref = host_func_raw_container.0.as_ref();

                log::trace!("Wrapper function context: {:?}", ctx_any_ref);

                let (exec_ctx_raw, (interpreter, fn_idx, ty)) = ctx_any_ref
                    .downcast_ref::<(*mut ExecutionContext, InterpreterInfo)>()
                    .unwrap();

                log::trace!("Wrapper function downcast successful");

                let mut interpreter = interpreter.borrow().clone();

                let exec_ctx = unsafe { &mut **exec_ctx_raw };

                exec_ctx.recursion_size += 1;

                let func_name = exec_ctx
                    .wasm_module
                    .meta
                    .exports
                    .find_function_name(*fn_idx)
                    .expect(&format!("function not found: {fn_idx}"))
                    .to_string();

                log::debug!("Wrapper function is for {func_name}");

                let args = slice::from_raw_parts(args_raw, ty.num_params())
                    .iter()
                    .cloned()
                    .zip(ty.params_iter())
                    .map(|(raw, typ)| Value::from_raw(raw, typ))
                    .collect();

                let rets = slice::from_raw_parts_mut(rets_raw, ty.num_results());

                let res = interpreter
                    .run(*fn_idx, args, *exec_ctx_raw)
                    .unwrap_or_trap(exec_ctx)
                    .into_iter()
                    .map(Value::into)
                    .collect::<Vec<_>>();

                log::debug!("wrapped func signature: {}", &ty);
                log::debug!("wrapped func res: {:?}", &res);

                debug_assert_eq!(ty.num_results(), res.len());

                rets.copy_from_slice(&res);
                exec_ctx.recursion_size -= 1;
                log::debug!("Wrapper function for {} finished", func_name);
            }

            let ty_idx =
                self.module.as_ref().unwrap().meta.functions[function_idx as usize].type_idx;
            let ty = self.module.as_ref().unwrap().meta.function_types[ty_idx as usize];

            // it's an Rc okay, let me clone it
            let info: InterpreterInfo = (self.interpreter.clone(), function_idx, ty);

            Ok(BoundaryFuncAndCtx {
                func,
                info: Some(info),
            })
        }

        fn get_internal_function_ptr(
            &self,
            function_idx: FuncIdx,
        ) -> Result<RawPointer, EngineError> {
            log::trace!(
                "Engine getting internal function pointer for function index {function_idx}"
            );
            // beyond cursed:
            let fn_idx_trans: &mut std::ffi::c_void =
                unsafe { transmute((function_idx + 1) as u64) };
            let non_null_ptr = NonNull::new(fn_idx_trans).unwrap();

            log::trace!("Engine got internal function pointer: {:?}", non_null_ptr);
            Ok(non_null_ptr)
        }

        fn get_global_value(&self, global_idx: GlobalIdx) -> Result<ValueRaw, EngineError> {
            self.interpreter
                .borrow_mut()
                .get_global_value(global_idx)
                .map_err(Into::into)
        }

        fn set_global_addr(&mut self, global_idx: GlobalIdx, addr: RawPointer) {
            self.interpreter
                .borrow_mut()
                .set_global_addr(global_idx, addr);
        }
    }
}

impl From<InterpreterError> for RuntimeError {
    fn from(interp_err: InterpreterError) -> Self {
        match interp_err {
            InterpreterError::StackExhausted => RuntimeError::Exhaustion,
            e @ _ => RuntimeError::EngineError(EngineError::InterpreterError(e)),
        }
    }
}
