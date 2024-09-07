use crate::{
    helper::{
        signals::SignalHandler,
        types::{WasmReturnType, WasmType, WasmTypeList},
        utils::macro_invoke_for_each_function_signature,
    },
    objects::execution_context::ExecutionContextWrapper,
    RuntimeError,
};
use cee_scape::call_with_sigsetjmp;
use core::ffi;
use ir::structs::value::{Value, ValueRaw};
use runtime_interface::ExecutionContext;
use std::{any::Any, mem::MaybeUninit, ptr::NonNull};
use wasi::WasiContext;
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

#[repr(transparent)]
pub struct HostFuncRawContainer(Box<dyn Any>);

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
        let container = Box::into_raw(Box::new(HostFuncRawContainer(Box::new(closure))));
        Function::from_host_func(container, ty, trampoline)
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
        let func_any_ref = unsafe { &(*callee_ctx.host_func_context).0 };
        let func = func_any_ref.downcast_ref::<Closure>().unwrap();
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
    pub wasi_context: *mut WasiContext,
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
pub struct Function(pub(crate) FunctionKind);

unsafe impl Sync for Function {}
unsafe impl Send for Function {}

#[derive(Clone)]
pub enum FunctionKind {
    /// A rust function made available to wasm code
    /// Calling Convention:
    ///     - BoundaryCC
    Host(BoundaryCCFuncTy, CalleeCtxt, FuncType),

    /// A generated and exported wasm function
    /// Calling Convention:
    ///     - BoundaryCC (for calls from rust code)
    Wasm(BoundaryCCFuncTy, CalleeCtxt, FuncType),

    /// A function that is part of the runtime
    /// Calling Convention:
    ///     - C CC (because we can hard-code parameter types)
    ///
    /// Note: Functions like `table_grow` and `table_fill` are runtime functions
    /// Note: The first argument is the current function's `ExecutionContext` parameter
    Runtime(NonNull<ffi::c_void>),

    Wasi(BoundaryCCFuncTy, FuncType),
}

impl Function {
    pub fn call(&self, params: &[Value]) -> Result<Vec<Value>, RuntimeError> {
        let (func, ctxt, ty) = match &self.0 {
            FunctionKind::Host(func, ctxt, ty) | FunctionKind::Wasm(func, ctxt, ty) => {
                (func, ctxt, ty)
            }
            FunctionKind::Runtime(..) => {
                return Err(RuntimeError::Msg(
                    "Internal runtime functions can only be called from wasm code".into(),
                ))
            }
            FunctionKind::Wasi(..) => {
                return Err(RuntimeError::Msg(
                    "Wasi runtime functions can only be called from wasm code".into(),
                ))
            }
        };
        let mut ret_values = vec![ValueRaw::u64(0); ty.num_results()];
        let params = params
            .iter()
            .cloned()
            .map(ValueRaw::from)
            .collect::<Vec<ValueRaw>>();

        let jmp_res = call_with_sigsetjmp(true, |jmp_buf| {
            ExecutionContextWrapper::set_trap_return_point(jmp_buf);
            SignalHandler::set_thread_executing_wasm();

            unsafe {
                func(*ctxt, params.as_ptr(), ret_values.as_mut_ptr());
            };

            SignalHandler::unset_thread_executing_wasm();
            0
        });
        if jmp_res != 0 {
            return Err(ExecutionContextWrapper::take_trap());
        }
        Ok(ret_values
            .iter()
            .zip(ty.results_iter())
            .map(|(val, val_type)| Value::from_raw(*val, val_type))
            .collect())
    }

    pub(crate) fn from_host_func(
        host_func_context: *const HostFuncRawContainer,
        ty: FuncType,
        ptr: BoundaryCCFuncTy,
    ) -> Self {
        Function(FunctionKind::Host(
            ptr,
            CalleeCtxt { host_func_context },
            ty,
        ))
    }

    pub(crate) fn from_wasm_func(
        execution_context: *mut ExecutionContext,
        ty: FuncType,
        boundary_func_ptr: BoundaryCCFuncTy,
    ) -> Self {
        Function(FunctionKind::Wasm(
            boundary_func_ptr,
            CalleeCtxt { execution_context },
            ty,
        ))
    }

    pub(crate) fn from_runtime_func(ptr: NonNull<ffi::c_void>) -> Self {
        Function(FunctionKind::Runtime(ptr))
    }

    pub(crate) fn from_wasi_func(boundary_func_ptr: BoundaryCCFuncTy, func_type: FuncType) -> Self {
        Function(FunctionKind::Wasi(boundary_func_ptr, func_type))
    }

    pub(crate) fn functype(&self) -> FuncType {
        match &self.0 {
            FunctionKind::Host(_, _, ty) | FunctionKind::Wasm(_, _, ty) => *ty,
            FunctionKind::Wasi(_, ty) => *ty,
            // we panic here, as this code path should never be taken (but it could, through a programmer error)
            FunctionKind::Runtime(_) => unreachable!("Runtime functions have no function type"),
        }
    }
}
