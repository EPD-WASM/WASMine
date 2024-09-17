use super::*;
use crate::parsable::ParseWithContext;

pub(crate) fn block(
    ctxt: &mut Context,
    i: &mut WasmBinaryReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let block_type = BlockType::parse_with_context(i, ctxt.module)?;
    o.write(module::instructions::Block { block_type });
    Ok(())
}
