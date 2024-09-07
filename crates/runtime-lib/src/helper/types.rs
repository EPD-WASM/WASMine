use super::utils::macro_invoke_for_each_function_signature;
use ir::structs::value::ValueRaw;
use std::mem::MaybeUninit;
use wasm_types::{FuncType, FuncTypeBuilder, NumType, ValType};

pub(crate) trait WasmType {
    fn valtype() -> ValType;
    unsafe fn from_raw(raw: &ValueRaw) -> Self;
    unsafe fn to_raw(self) -> ValueRaw;
}

macro_rules! wasm_type_for_integers {
    ($($integer_ty:ident, $extraction_func:ident, $val_type:ident)*) => ($(
        impl WasmType for $integer_ty {
            fn valtype() -> ValType {
                ValType::Number(NumType::$val_type)
            }

            unsafe fn from_raw(raw: &ValueRaw) -> Self {
                raw.$extraction_func()
            }

            unsafe fn to_raw(self) -> ValueRaw {
                ValueRaw::from(self)
            }
        }
    )*)
}

wasm_type_for_integers! {
    i32, as_i32, I32
    i64, as_i64, I64
    u32, as_u32, I32
    u64, as_u64, I64
}

macro_rules! wasm_type_for_floats {
    ($($float_ty:ident, $extraction_func:ident, $val_type:ident)*) => ($(
        impl WasmType for $float_ty {
            fn valtype() -> ValType {
                ValType::Number(NumType::$val_type)
            }

            unsafe fn from_raw(raw: &ValueRaw) -> Self {
                $float_ty::from_bits(raw.$extraction_func())
            }

            unsafe fn to_raw(self) -> ValueRaw {
                ValueRaw::from(self)
            }
        }
    )*)
}

wasm_type_for_floats! {
    f32, as_f32, F32
    f64, as_f64, F64
}

pub(crate) trait WasmTypeList {
    fn valtypes() -> impl Iterator<Item = ValType>;
    unsafe fn from_raw(values: &[MaybeUninit<ValueRaw>]) -> Self;
}

macro_rules! impl_wasm_ty_list {
    ($num:tt $($args:ident)*) => (
        // define WasmTypeList impls for all tuples of wasm types (0..16)
        impl<$($args),*> WasmTypeList for ($($args,)*)
        where
            $($args: WasmType,)*
        {
            fn valtypes() -> impl Iterator<Item = ValType> {
                IntoIterator::into_iter([$($args::valtype(),)*])
            }

            // the compiler does not recognize that we return a tuple and therefore
            // complains about unused stuff -> remove when future compilers are smarter
            #[allow(clippy::unused_unit,unused_assignments,unused_mut)]
            unsafe fn from_raw(values: &[MaybeUninit<ValueRaw>]) -> Self {
                let mut idx = 0;
                (
                    $({
                        debug_assert!(idx < values.len());
                        let raw = values.get_unchecked(idx).assume_init_ref();
                        idx += 1;
                        $args::from_raw(raw)
                    },)*
                )
            }
        }
    );
}
macro_invoke_for_each_function_signature!(impl_wasm_ty_list);

pub(crate) trait WasmReturnType {
    fn func_type(params: impl Iterator<Item = ValType>) -> FuncType;
    unsafe fn to_raw(self, ret_args: *mut ValueRaw);
}

macro_rules! wasm_return_type_impl {
    // for one return parameter => no tuple return
    ($n:tt $t:ident) => (
        #[allow(non_snake_case)]
        impl<$t> WasmReturnType for $t
        where
            $t: WasmType
        {
            fn func_type(params: impl Iterator<Item = ValType>) -> FuncType {
                let mut builder = FuncTypeBuilder::new();
                for param in params {
                    builder = builder.add_param(param);
                }
                builder = builder.add_result($t::valtype());
                builder.finish()
            }

            unsafe fn to_raw(self, ret_args: *mut ValueRaw) {
                *ret_args = self.to_raw();
            }
        }
    );

    // zero or multiple return params => tuple return
    ($n:tt $($t:ident)*) => (
        #[allow(non_snake_case)]
        impl<$($t),*> WasmReturnType for ($($t,)*)
        where
            $($t: WasmType,)*
        {
            fn func_type(params: impl Iterator<Item = ValType>) -> FuncType {
                let mut builder = FuncTypeBuilder::new();
                for param in params {
                    builder = builder.add_param(param);
                }
                $(
                    builder = builder.add_result($t::valtype());
                )*
                builder.finish()
            }

            #[allow(unused_assignments)]
            unsafe fn to_raw(self, ret_args: *mut ValueRaw) {
                let ($($t,)*) = self; // unpack tuple

                #[allow(unused_mut)]
                let mut idx = 0;
                $(
                    ret_args.add(idx).write($t::to_raw($t));
                    idx += 1;
                )*
            }
        }
    );
}

macro_invoke_for_each_function_signature!(wasm_return_type_impl);
