use super::*;
use wasm_types::LabelIdx;

pub(crate) fn br_if(
    _: &mut Context,
    i: &mut WasmBinaryReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let label_idx = LabelIdx::parse(i)?;
    o.write(BrIf { label_idx });
    Ok(())
}
