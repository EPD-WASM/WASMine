use crate::{
    execution_context::ExecutionContextWrapper,
    signals::SignalHandler,
    types::{WasmReturnType, WasmType, WasmTypeList},
    utils::macro_invoke_for_each_function_signature,
    RuntimeError,
};
use cee_scape::call_with_sigsetjmp;
use core::ffi;
use ir::structs::value::{Value, ValueRaw};
use runtime_interface::ExecutionContext;
use std::{any::Any, mem::MaybeUninit, ptr::NonNull};
use wasm_types::FuncType;

pub trait IntoFunc<Params, Returns> {
    fn into_func(self) -> Function;
}

macro_rules! impl_into_func_trait {
    ($num:tt $($args:ident)*) => {
        // impl for function taking x single parameters
        #[allow(non_snake_case)]
        impl<Func, $($args,)* Returns> IntoFunc<($($args,)*), Returns> for Func
        where
            Func: Fn($($args),*) -> Returns + 'static,
            $($args: WasmType,)*
            Returns: WasmReturnType,
        {
            fn into_func(self) -> Function {
                HostFuncRawContainer::from_closure(move |($($args,)*),| {
                    self($( $args ),*)
                })
            }
        }
    }
}
macro_invoke_for_each_function_signature!(impl_into_func_trait);

pub struct HostFuncRaw<F> {
    closure: F,
}

pub struct HostFuncRawContainer {
    func: Box<dyn Any>,
}

type TestCall = unsafe extern "C" fn(*mut ExecutionContext, *const ValueRaw, *const ValueRaw) -> ();

impl HostFuncRawContainer {
    fn from_closure<Closure, ParamTypes, RetType>(closure: Closure) -> Function
    where
        Closure: Fn(ParamTypes) -> RetType + 'static,
        ParamTypes: WasmTypeList,
        RetType: WasmReturnType,
    {
        let ty = RetType::func_type(ParamTypes::valtypes());
        let trampoline: BoundaryCCFuncTy =
            Self::host_func_trampoline::<Closure, ParamTypes, RetType>;
        let container = Box::new(HostFuncRawContainer {
            func: Box::from(HostFuncRaw { closure }),
        });
        Function::from_host_func(Box::into_raw(container), ty, trampoline)
    }

    unsafe extern "C" fn host_func_trampoline<Closure, ParamTypes, RetType>(
        callee_ctx: CalleeCtxt,
        args: *const ValueRaw,
        ret_args: *mut ValueRaw,
    ) where
        Closure: Fn(ParamTypes) -> RetType + 'static,
        ParamTypes: WasmTypeList,
        RetType: WasmReturnType,
    {
        let args = core::slice::from_raw_parts(
            args.cast::<MaybeUninit<ValueRaw>>(),
            ParamTypes::valtypes().count(),
        );
        let hfc = &*(callee_ctx.host_func_context);
        let hf = hfc.func.downcast_ref::<HostFuncRaw<Closure>>().unwrap();
        let func = &hf.closure;
        let params = ParamTypes::from_raw(args);
        let ret = func(params);
        RetType::to_raw(ret, ret_args);
    }

    pub fn wrap<Params, Returns>(closure: impl IntoFunc<Params, Returns>) -> Function {
        closure.into_func()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FunctionError {
    #[error("function not callable from rust code")]
    MissingRustCallConvPtr,
}

#[derive(Clone, Copy)]
#[repr(C)]
pub union CalleeCtxt {
    pub execution_context: *mut ExecutionContext,
    pub host_func_context: *const HostFuncRawContainer,
    // placeholder:
    pub wasi_context: *const ffi::c_void,
}

/// The "boundary calling convention" extends the C calling convention with a few additional rules:
/// - The first argument is the callee context (some context required for function execution)
/// - The second argument is a pointer to the array of arguments
/// - The third argument is a pointer to the array of return values
/// - The function returns nothing
///
/// The boundary calling convention is used for calls from wasm code to the runtime rust code and vice versa.
pub(crate) type BoundaryCCFuncTy =
    unsafe extern "C" fn(CalleeCtxt, *const ValueRaw, *mut ValueRaw) -> ();

#[derive(Clone)]
pub struct Function {
    pub(crate) kind: FunctionKind,
    pub(crate) ty: FuncType,
}

unsafe impl Sync for Function {}
unsafe impl Send for Function {}

#[derive(Clone, Copy)]
pub enum FunctionKind {
    /// A rust function made available to wasm code
    /// Calling Convention:
    ///     - BoundaryCC
    Host(BoundaryCCFuncTy, CalleeCtxt),

    /// A generated and exported wasm function
    /// Calling Convention:
    ///     - BoundaryCC (for calls from rust code)
    ///     - WasmCC (for calls from wasm code)
    ///         (TODO: this effectively prevents calling interpreted functions from llvm code,
    ///                because the interpreter can not dynamically generated arbitrary function
    ///                signatures)
    Wasm(BoundaryCCFuncTy, NonNull<ffi::c_void>, CalleeCtxt),

    /// A function that is part of the runtime
    /// Calling Convention:
    ///     - C CC (because we can hard-code parameter types)
    ///
    /// Note: Functions like `table_grow` and `table_fill` are runtime functions
    /// Note: The first argument is the current function's `ExecutionContext` parameter
    Runtime(NonNull<ffi::c_void>),
}

impl Function {
    pub fn call(&self, params: &[Value]) -> Result<Vec<Value>, RuntimeError> {
        let mut ret_values = vec![ValueRaw::u64(0); self.ty.1.len()];
        let params = params
            .iter()
            .cloned()
            .map(ValueRaw::from)
            .collect::<Vec<ValueRaw>>();
        let (func, ctxt) = match self.kind {
            FunctionKind::Host(func, ctxt) => (func, ctxt),
            FunctionKind::Wasm(func, _, ctxt) => (func, ctxt),
            FunctionKind::Runtime(_) => {
                return Err(RuntimeError::Msg(
                    "Internal runtime functions can only be called from wasm code".into(),
                ))
            }
        };
        let jmp_res = call_with_sigsetjmp(true, |jmp_buf| {
            ExecutionContextWrapper::set_trap_return_point(jmp_buf);
            SignalHandler::set_thread_executing_wasm();

            unsafe {
                func(ctxt, params.as_ptr(), ret_values.as_mut_ptr());
            };

            SignalHandler::unset_thread_executing_wasm();
            0
        });
        if jmp_res != 0 {
            return Err(RuntimeError::Trap(ExecutionContextWrapper::take_trap_msg()));
        }
        Ok(ret_values
            .iter()
            .zip(self.ty.1.iter())
            .map(|(val, val_type)| Value::from_raw(*val, *val_type))
            .collect())
    }

    pub(crate) fn from_host_func(
        host_func_context: *mut HostFuncRawContainer,
        ty: FuncType,
        ptr: BoundaryCCFuncTy,
    ) -> Self {
        Function {
            kind: FunctionKind::Host(ptr, CalleeCtxt { host_func_context }),
            ty,
        }
    }

    pub(crate) fn from_wasm_func(
        execution_context: *mut ExecutionContext,
        ty: FuncType,
        boundary_func_ptr: BoundaryCCFuncTy,
        wasm_func_ptr: NonNull<ffi::c_void>,
    ) -> Self {
        Function {
            kind: FunctionKind::Wasm(
                boundary_func_ptr,
                wasm_func_ptr,
                CalleeCtxt { execution_context },
            ),
            ty,
        }
    }

    pub(crate) fn from_runtime_func(ty: FuncType, ptr: NonNull<ffi::c_void>) -> Self {
        Function {
            kind: FunctionKind::Runtime(ptr),
            ty,
        }
    }
}
