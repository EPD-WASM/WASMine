use super::*;
use crate::parsable::ParseWithContext;

pub(crate) fn block(
    ctxt: &mut Context,
    i: &mut WasmBinaryReader,
    o: &mut dyn InstructionConsumer,
) -> ParseResult {
    let block_type = BlockType::parse_with_context(i, ctxt.module)?;
    o.write_block(module::instructions::Block { block_type });
    Ok(())
}
