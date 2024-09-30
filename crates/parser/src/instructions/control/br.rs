use super::*;
use wasm_types::LabelIdx;

pub(crate) fn br(
    _: &mut Context,
    i: &mut WasmBinaryReader,
    o: &mut dyn InstructionConsumer,
) -> ParseResult {
    let label_idx = LabelIdx::parse(i)?;
    o.write_br(Br { label_idx });
    Ok(())
}
