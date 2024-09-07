mod control;
mod memory;
pub mod meta;
mod numeric;
mod parametric;
mod reference;
mod table;
mod variable;

pub use control::*;
pub use memory::*;
pub use meta::*;
pub use numeric::*;
pub use parametric::*;
pub use reference::*;
pub use table::*;
pub use variable::*;

use crate::{DecodingError, InstructionDecoder, InstructionEncoder};
use std::fmt::{self, Display, Formatter};
use wasm_types::*;

pub trait Instruction {
    fn serialize(self, o: &mut InstructionEncoder);
    fn deserialize(_: &mut InstructionDecoder, _t: InstructionType) -> Result<Self, DecodingError>
    where
        Self: std::marker::Sized,
    {
        unimplemented!()
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Variable {
    pub type_: ValType,
    pub id: VariableID,
}

pub type VariableID = u32;

macro_rules! extract_numtype {
    ($val_type:expr) => {
        match $val_type {
            ValType::Number(t) => t,
            _ => return Err(DecodingError::TypeMismatch),
        }
    };
}
pub(crate) use extract_numtype;
