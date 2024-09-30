use super::*;
use crate::parsable::ParseWithContext;

pub(crate) fn r#loop(
    ctxt: &mut Context,
    i: &mut WasmBinaryReader,
    o: &mut dyn InstructionConsumer,
) -> ParseResult {
    let block_type = BlockType::parse_with_context(i, ctxt.module)?;
    o.write_loop(Loop { block_type });
    Ok(())
}
