use super::*;
use wasm_types::LabelIdx;

pub(crate) fn br(
    _: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let label_idx = LabelIdx::parse(i)?;
    o.write(Br { label_idx });
    Ok(())
}
