pub(crate) mod block;
pub(crate) mod br;
pub(crate) mod br_if;
pub(crate) mod br_table;
pub(crate) mod call;
pub(crate) mod call_indirect;
pub(crate) mod if_else;
pub(crate) mod r#loop;
pub(crate) mod nop;
pub(crate) mod pseudo;
pub(crate) mod r#return;
pub(crate) mod unreachable;

pub(crate) use block::*;
pub(crate) use br::*;
pub(crate) use br_if::*;
pub(crate) use br_table::*;
pub(crate) use call::*;
pub(crate) use call_indirect::*;
pub(crate) use if_else::*;
pub(crate) use nop::*;
pub(crate) use pseudo::*;
pub(crate) use r#loop::*;
pub(crate) use r#return::*;
pub(crate) use unreachable::*;

use super::*;
use crate::parser::parsable::Parse;
use crate::structs::instruction::ControlInstruction;
use crate::wasm_types::BlockType;

#[derive(Debug, Clone)]
pub(crate) struct Block {
    block_type: BlockType,
}
