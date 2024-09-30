use super::*;

// pseudo instructions required for basic block parsing

pub(crate) fn r#else(
    _: &mut Context,
    _: &mut WasmBinaryReader,
    o: &mut dyn InstructionConsumer,
) -> ParseResult {
    o.write_else();
    Ok(())
}

pub(crate) fn end(
    _: &mut Context,
    _: &mut WasmBinaryReader,
    o: &mut dyn InstructionConsumer,
) -> ParseResult {
    o.write_end();
    Ok(())
}
