// https://webassembly.github.io/spec/core/exec/runti`me`.html

use crate::utils::numeric_transmutes::{Bit32, Bit64};
use core::ffi;
use std::fmt::{Display, Formatter};
use wasm_types::{FuncIdx, GlobalIdx, NumType, RefType, ValType};

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
    Extern(*const ffi::c_void),
}

impl Display for Reference {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Reference::Null => write!(f, "null"),
            Reference::Function(idx) => write!(f, "func[{}]", *idx as u64),
            Reference::Extern(idx) => write!(f, "extern[{}]", *idx as u64),
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
                Value::Reference(Reference::Extern(val as _))
            }
            ValType::Reference(RefType::FunctionReference) => {
                Value::Reference(Reference::Function(val as _))
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
            Value::Reference(Reference::Function(idx)) => *idx as u64,
            Value::Reference(Reference::Extern(idx)) => *idx as u64,
            Value::Reference(Reference::Null) => u32::MAX as u64,
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

#[derive(Debug, Clone, PartialEq)]
pub enum ConstantValue {
    V(Value),
    // we can't resolve the value of imported globals at parsing time
    Global(GlobalIdx),
    // we can't resolve the function pointer at parsing time
    FuncPtr(FuncIdx),
}
