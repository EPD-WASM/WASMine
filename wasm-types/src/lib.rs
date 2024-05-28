pub mod instruction;
pub mod module;

use std::fmt::{self, Display, Formatter};

pub use instruction::*;
pub use module::*;

/// https://webassembly.github.io/spec/core/syntax/types.html#number-types
#[derive(Debug, Clone, PartialEq, Copy, Default)]
pub enum NumType {
    #[default]
    I32,
    I64,
    F32,
    F64,
}

/// https://webassembly.github.io/spec/core/syntax/types.html#reference-types
#[derive(Debug, Clone, PartialEq, Copy, Default)]
pub enum RefType {
    #[default]
    FunctionReference,
    ExternReference,
}

/// https://webassembly.github.io/spec/core/syntax/types.html#value-types
#[derive(Debug, Clone, PartialEq, Default, Copy)]
pub enum ValType {
    Number(NumType),
    Reference(RefType),
    /// https://webassembly.github.io/spec/core/syntax/types.html#vector-types
    #[default]
    VecType,
}

/// https://webassembly.github.io/spec/core/syntax/types.html#result-types
pub type ResType = Vec<ValType>;

// https://webassembly.github.io/spec/core/syntax/types.html#result-types
pub type FuncType = (ResType, ResType);

#[derive(Debug, Clone, Copy)]
pub struct LimType {
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
            ValType::Number(nt) => write!(f, "{}", nt),
            ValType::Reference(rt) => write!(f, "{}", rt),
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
