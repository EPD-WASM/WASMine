pub(crate) mod basic_block;
mod decoder;
mod encoder;
pub(crate) mod function;

pub(crate) use decoder::{DecodingError, InstructionDecoder};
pub(crate) use encoder::InstructionEncoder;

use crate::instructions::*;
use crate::structs::instruction::ControlInstruction;
use std::collections::VecDeque;
use wasm_types::*;

pub(crate) struct IR {
    pub(crate) functions: Vec<function::Function>,
}
