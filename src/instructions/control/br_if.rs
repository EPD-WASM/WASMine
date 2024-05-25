use super::*;
use wasm_types::LabelIdx;

#[derive(Debug, Clone)]
pub(crate) struct BrIf {
    label_idx: LabelIdx,
}

impl Instruction for BrIf {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.finish(ControlInstruction::BrIf(self.label_idx));
    }

    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        Ok(BrIf {
            label_idx: i.read_immediate()?,
        })
    }
}

pub(crate) fn br_if(
    _: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let label_idx = LabelIdx::parse(i)?;
    o.write(BrIf { label_idx });
    Ok(())
}
