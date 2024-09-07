use super::*;
use wasm_types::{TableIdx, TypeIdx};

pub(crate) fn call_indirect(
    _: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let type_idx = TypeIdx::parse(i)?;
    let table_idx = TableIdx::parse(i)?;
    o.write(CallIndirect {
        type_idx,
        table_idx,
    });
    Ok(())
}
