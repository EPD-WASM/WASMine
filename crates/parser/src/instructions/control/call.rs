use super::*;
use wasm_types::FuncIdx;

pub(crate) fn call(
    _: &mut Context,
    i: &mut WasmBinaryReader,
    o: &mut dyn InstructionConsumer,
) -> ParseResult {
    let func_idx = FuncIdx::parse(i)?;
    o.write_call(Call { func_idx });
    Ok(())
}
