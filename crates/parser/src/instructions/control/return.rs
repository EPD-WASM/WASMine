use super::*;

pub(crate) fn r#return(
    _: &mut Context,
    _: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    o.write(Return {});
    Ok(())
}
