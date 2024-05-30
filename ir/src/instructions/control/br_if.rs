use super::*;
use wasm_types::LabelIdx;

#[derive(Debug, Clone)]
pub struct BrIf {
    pub label_idx: LabelIdx,
}

impl Instruction for BrIf {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.finish(ControlInstruction::BrIf(self.label_idx));
    }

    fn deserialize(_: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        unimplemented!(
            "Control instructions are not serialized and can therefore not be deserialized."
        )
    }
}
