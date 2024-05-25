use super::*;
use wasm_types::FuncIdx;

#[derive(Debug, Clone)]
pub(crate) struct Call {
    func_idx: FuncIdx,
}

impl Instruction for Call {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.finish(ControlInstruction::Call(self.func_idx));
    }

    fn deserialize(_: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        unimplemented!(
            "Control instructions are not serialized and can therefore not be deserialized."
        )
    }
}

pub(crate) fn call(
    _: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let func_idx = FuncIdx::parse(i)?;
    o.write(Call { func_idx });
    Ok(())
}
