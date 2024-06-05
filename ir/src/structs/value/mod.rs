// https://webassembly.github.io/spec/core/exec/runti`me`.html

use std::fmt::{Display, Formatter};

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

impl Display for Number {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Number::I32(n) => write!(f, "{}", n),
            Number::I64(n) => write!(f, "{}", n),
            Number::U32(n) => write!(f, "{}", n),
            Number::U64(n) => write!(f, "{}", n),
            Number::S32(n) => write!(f, "{}", n),
            Number::S64(n) => write!(f, "{}", n),
            Number::F32(n) => write!(f, "{}", n),
            Number::F64(n) => write!(f, "{}", n),
        }
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

impl Display for Reference {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Reference::Null => write!(f, "null"),
            Reference::Function(idx) => write!(f, "func[{}]", idx),
            Reference::Extern(idx) => write!(f, "extern[{}]", idx),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Number(Number),
    Vector(Vector),
    Reference(Reference),
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", *n),
            Value::Vector(v) => write!(f, "{}", v),
            Value::Reference(r) => write!(f, "{}", *r),
        }
    }
}
