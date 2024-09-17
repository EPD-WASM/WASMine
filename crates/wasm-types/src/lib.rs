mod functypes;
mod instruction;
mod module;

use rkyv::{Deserialize, Serialize, Archive};
use std::fmt::{self, Display, Formatter};

pub use functypes::{FuncType, FuncTypeBuilder};
pub use instruction::*;
pub use module::*;

/// https://webassembly.github.io/spec/core/syntax/types.html#number-types
#[derive(Debug, Clone, PartialEq, Copy, Default, Eq, Hash, Archive, Deserialize, Serialize)]
pub enum NumType {
    #[default]
    I32,
    I64,
    F32,
    F64,
}

/// https://webassembly.github.io/spec/core/syntax/types.html#reference-types
#[derive(Debug, Clone, PartialEq, Copy, Default, Eq, Hash, Archive, Deserialize, Serialize)]
pub enum RefType {
    #[default]
    FunctionReference,
    ExternReference,
}

/// https://webassembly.github.io/spec/core/syntax/types.html#value-types
#[derive(Debug, Clone, PartialEq, Default, Copy, Eq, Hash, Archive, Deserialize, Serialize)]
pub enum ValType {
    Number(NumType),
    Reference(RefType),
    /// https://webassembly.github.io/spec/core/syntax/types.html#vector-types
    #[default]
    VecType,
}

impl ValType {
    #[inline]
    pub const fn i32() -> Self {
        ValType::Number(NumType::I32)
    }

    #[inline]
    pub const fn i64() -> Self {
        ValType::Number(NumType::I64)
    }

    #[inline]
    pub const fn f32() -> Self {
        ValType::Number(NumType::F32)
    }

    #[inline]
    pub const fn f64() -> Self {
        ValType::Number(NumType::F64)
    }

    #[inline]
    pub const fn funcref() -> Self {
        ValType::Reference(RefType::FunctionReference)
    }

    #[inline]
    pub const fn externref() -> Self {
        ValType::Reference(RefType::ExternReference)
    }

    #[inline]
    pub const fn vec() -> Self {
        ValType::VecType
    }
}

/// https://webassembly.github.io/spec/core/syntax/types.html#result-types
pub type ResType = Vec<ValType>;

#[derive(Debug, Clone, Copy, PartialEq, Archive, Deserialize, Serialize)]
pub struct Limits {
    pub min: u32,
    pub max: Option<u32>,
}

pub type TypeIdx = u32;
pub type FuncIdx = u32;
pub type TableIdx = u32;
pub type MemIdx = u32;
pub type GlobalIdx = u32;
pub type ElemIdx = u32;
pub type DataIdx = u32;
pub type LocalIdx = u32;
pub type LabelIdx = u32;

impl ValType {
    pub fn is_valtype_byte(byte: u8) -> bool {
        matches!(byte, 0x7F | 0x7E | 0x7D | 0x7C | 0x7B | 0x70 | 0x6F)
    }
}

impl Display for ValType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ValType::Number(nt) => write!(f, "{nt}"),
            ValType::Reference(rt) => write!(f, "{rt}"),
            ValType::VecType => write!(f, "vec"),
        }
    }
}

impl Display for NumType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            NumType::I32 => write!(f, "i32"),
            NumType::I64 => write!(f, "i64"),
            NumType::F32 => write!(f, "f32"),
            NumType::F64 => write!(f, "f64"),
        }
    }
}

impl Display for RefType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            RefType::FunctionReference => write!(f, "funcref"),
            RefType::ExternReference => write!(f, "externref"),
        }
    }
}
