use super::*;
use wasm_types::LabelIdx;

#[derive(Debug, Clone)]
pub(crate) struct Br {
    label_idx: LabelIdx,
}

impl Instruction for Br {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.finish(ControlInstruction::Br(self.label_idx));
    }

    fn deserialize(_: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        unimplemented!(
            "Control instructions are not serialized and can therefore not be deserialized."
        )
    }
}

pub(crate) fn br(
    _: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let label_idx = LabelIdx::parse(i)?;
    o.write(Br { label_idx });
    Ok(())
}
