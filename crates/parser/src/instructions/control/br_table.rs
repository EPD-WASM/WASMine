use super::*;
use wasm_types::LabelIdx;

pub(crate) fn br_table(
    _: &mut Context,
    i: &mut WasmBinaryReader,
    o: &mut dyn InstructionConsumer,
) -> ParseResult {
    let num_labels = i.read_leb128::<u32>()?;
    let label_indices = (0..num_labels)
        .map(|_| LabelIdx::parse(i))
        .collect::<Result<Vec<LabelIdx>, ParserError>>()?;
    let default_label_idx = LabelIdx::parse(i)?;
    o.write_br_table(BrTable {
        label_indices,
        default_label_idx,
    });
    Ok(())
}
