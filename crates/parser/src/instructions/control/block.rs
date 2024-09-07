use super::*;
use crate::parsable::ParseWithContext;

pub(crate) fn block(
    ctxt: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let block_type = BlockType::parse_with_context(i, ctxt.module)?;
    o.write(ir::instructions::Block { block_type });
    Ok(())
}
