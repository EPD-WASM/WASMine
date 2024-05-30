mod control;
mod memory;
pub(crate) mod meta;
mod numeric;
mod parametric;
mod reference;
mod table;
mod variable;

pub(crate) use control::*;
pub(crate) use memory::*;
pub(crate) use numeric::*;
pub(crate) use parametric::*;
pub(crate) use reference::*;
pub(crate) use table::*;
pub(crate) use variable::*;

use crate::parsable::Parse;
use crate::{
    wasm_stream_reader::WasmStreamReader, Context, ParseResult, ParserError, ValidationError,
};
use ir::instructions::*;
use ir::structs::instruction::ControlInstruction;
use ir::InstructionEncoder;
use wasm_types::*;

type C<'a> = Context<'a>;
type I<'a> = WasmStreamReader<'a>;
type O = InstructionEncoder;
type PR = ParseResult;
