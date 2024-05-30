// https://webassembly.github.io/spec/core/exec/runti`me`.html

use wasm_types::FuncIdx;
use std::
    rc::Rc
;

mod number_ops;
mod number_impls;

#[derive(Debug, Clone)]
pub(crate) enum Number {
    I32(u32),
    I64(u64),
    U32(u32),
    U64(u64),
    S32(i32),
    S64(i64),
    F32(f32),
    F64(f64),
}

impl Default for Number {
    fn default() -> Self {
        Number::I32(0)
    }
}

pub(crate) type Vector = u128;

pub(crate) type FunctionReference = FuncIdx;
pub(crate) type ExternReference = u32;

#[derive(Debug, Clone)]
pub(crate) enum Reference {
    Null,
    Function(FuncIdx),
    Extern(u32),
}

#[derive(Debug, Clone)]
pub(crate) enum Value {
    Number(Number),
    Vector(Vector),
    Reference(Reference),
}
