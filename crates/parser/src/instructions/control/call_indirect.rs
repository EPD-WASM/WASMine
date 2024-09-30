use super::*;
use wasm_types::{TableIdx, TypeIdx};

pub(crate) fn call_indirect(
    _: &mut Context,
    i: &mut WasmBinaryReader,
    o: &mut dyn InstructionConsumer,
) -> ParseResult {
    let type_idx = TypeIdx::parse(i)?;
    let table_idx = TableIdx::parse(i)?;
    o.write_call_indirect(CallIndirect {
        type_idx,
        table_idx,
    });
    Ok(())
}
