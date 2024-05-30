use super::*;
use wasm_types::FuncIdx;

pub(crate) fn call(
    _: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let func_idx = FuncIdx::parse(i)?;
    o.write(Call { func_idx });
    Ok(())
}
