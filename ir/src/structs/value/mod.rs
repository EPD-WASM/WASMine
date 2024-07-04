// https://webassembly.github.io/spec/core/exec/runti`me`.html

use std::fmt::{Display, Formatter};

use wasm_types::{FuncIdx, NumType, RefType, ValType};

use crate::utils::numeric_transmutes::{Bit32, Bit64};

mod number_impls;
mod number_ops;

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
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

impl Value {
    pub fn from_generic(val_type: ValType, val: u64) -> Self {
        match val_type {
            ValType::Number(NumType::I32) => Value::Number(Number::I32(val.trans_u32())),
            ValType::Number(NumType::I64) => Value::Number(Number::I64(val)),
            ValType::Number(NumType::F32) => Value::Number(Number::F32(val.trans_f32())),
            ValType::Number(NumType::F64) => Value::Number(Number::F64(val.trans_f64())),
            ValType::Reference(RefType::ExternReference) => {
                Value::Reference(Reference::Extern(val.trans_u32()))
            }
            ValType::Reference(RefType::FunctionReference) => {
                Value::Reference(Reference::Function(val.trans_u32()))
            }
            ValType::VecType => Value::Vector(val.trans_u64() as u128),
        }
    }

    pub fn to_generic(&self) -> u64 {
        match self {
            Value::Number(Number::I32(n)) => n.trans_u64(),
            Value::Number(Number::I64(n)) => n.trans_u64(),
            Value::Number(Number::U32(n)) => n.trans_u64(),
            Value::Number(Number::U64(n)) => n.trans_u64(),
            Value::Number(Number::S32(n)) => n.trans_u64(),
            Value::Number(Number::S64(n)) => n.trans_u64(),
            Value::Number(Number::F32(n)) => n.trans_u64(),
            Value::Number(Number::F64(n)) => n.trans_u64(),
            Value::Vector(_) => unimplemented!(),
            Value::Reference(Reference::Function(idx)) => idx.trans_u64(),
            Value::Reference(Reference::Extern(idx)) => idx.trans_u64(),
            Value::Reference(Reference::Null) => 0,
        }
    }

    pub fn r#type(&self) -> ValType {
        match self {
            Value::Number(Number::I32(_))
            | Value::Number(Number::U32(_))
            | Value::Number(Number::S32(_)) => ValType::Number(NumType::I32),
            Value::Number(Number::I64(_))
            | Value::Number(Number::U64(_))
            | Value::Number(Number::S64(_)) => ValType::Number(NumType::I64),
            Value::Number(Number::F32(_)) => ValType::Number(NumType::F32),
            Value::Number(Number::F64(_)) => ValType::Number(NumType::F64),
            Value::Vector(_) => ValType::VecType,
            Value::Reference(Reference::Function(_)) => {
                ValType::Reference(RefType::FunctionReference)
            }
            Value::Reference(Reference::Extern(_)) => ValType::Reference(RefType::ExternReference),
            Value::Reference(Reference::Null) => ValType::Reference(RefType::FunctionReference),
        }
    }
}
