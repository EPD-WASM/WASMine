use super::*;
use crate::parsable::ParseWithContext;

pub(crate) fn r#loop(
    ctxt: &mut Context,
    i: &mut WasmBinaryReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let block_type = BlockType::parse_with_context(i, ctxt.module)?;
    o.write(Loop { block_type });
    Ok(())
}
