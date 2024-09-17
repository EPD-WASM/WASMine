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

use crate::ir::context::Context;
use crate::parsable::Parse;
use crate::{wasm_stream_reader::WasmBinaryReader, ParseResult, ParserError, ValidationError};
use module::instructions::*;
use module::objects::instruction::ControlInstruction;
use module::InstructionEncoder;
use wasm_types::*;

type C<'a> = Context<'a>;
type I<'a> = WasmBinaryReader<'a>;
type O = InstructionEncoder;
type PR = ParseResult;
