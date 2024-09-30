use super::*;

pub(crate) fn nop(
    _: &mut Context,
    _: &mut WasmBinaryReader,
    _: &mut dyn InstructionConsumer,
) -> ParseResult {
    // we don't terminate parsing for this one. Why is "nop" a control instruction anyways?
    Ok(())
}
