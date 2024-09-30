use super::*;

pub(crate) fn r#return(
    _: &mut Context,
    _: &mut WasmBinaryReader,
    o: &mut dyn InstructionConsumer,
) -> ParseResult {
    o.write_return();
    Ok(())
}
