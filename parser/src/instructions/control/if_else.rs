use super::*;
use crate::parsable::ParseWithContext;

pub(crate) fn if_else(
    ctxt: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let block_type = BlockType::parse_with_context(i, ctxt.module)?;
    o.write(IfElse { block_type });
    Ok(())
}
