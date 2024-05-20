mod control;
mod memory;
pub(crate) mod meta;
mod numeric;
mod parametric;
mod reference;
pub(crate) mod storage;
mod table;
mod variable;

pub(crate) use control::*;
pub(crate) use memory::*;
pub(crate) use numeric::*;
pub(crate) use parametric::*;
pub(crate) use reference::*;
pub(crate) use table::*;
pub(crate) use variable::*;

use self::storage::{DecodingError, InstructionDecoder, InstructionEncoder};
use crate::{
    parser::{
        wasm_stream_reader::WasmStreamReader, Context, ParseResult, ParserError, ValidationError,
    },
    wasm_types::{InstructionType, ValType},
};

type C<'a> = Context<'a>;
type I<'a> = WasmStreamReader<'a>;
type O = InstructionEncoder;
type PR = ParseResult;

pub(crate) trait Instruction {
    fn serialize(self, o: &mut InstructionEncoder);
    fn deserialize(_: &mut InstructionDecoder, _t: InstructionType) -> Result<Self, DecodingError>
    where
        Self: std::marker::Sized,
    {
        unimplemented!()
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct Variable {
    pub(crate) type_: ValType,
    pub(crate) id: VariableID,
}

pub(crate) type VariableID = u32;

macro_rules! extract_numtype {
    ($val_type:expr) => {
        match $val_type {
            ValType::Number(t) => t,
            _ => return Err(DecodingError::TypeMismatch),
        }
    };
}
pub(crate) use extract_numtype;
