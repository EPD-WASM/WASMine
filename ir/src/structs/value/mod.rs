// https://webassembly.github.io/spec/core/exec/runti`me`.html

use std::rc::Rc;
use wasm_types::FuncIdx;

mod number_impls;
mod number_ops;

#[derive(Debug, Clone)]
pub enum Number {
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

pub type Vector = u128;

pub type FunctionReference = FuncIdx;
pub type ExternReference = u32;

#[derive(Debug, Clone)]
pub enum Reference {
    Null,
    Function(FuncIdx),
    Extern(u32),
}

#[derive(Debug, Clone)]
pub enum Value {
    Number(Number),
    Vector(Vector),
    Reference(Reference),
}
