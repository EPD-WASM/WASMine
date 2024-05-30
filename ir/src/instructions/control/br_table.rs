use super::*;
use wasm_types::LabelIdx;

#[derive(Debug, Clone)]
pub struct BrTable {
    pub label_indices: Vec<LabelIdx>,
    pub default_label_idx: LabelIdx,
}

impl Instruction for BrTable {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.finish(ControlInstruction::BrTable(
            self.default_label_idx,
            self.label_indices,
        ));
    }

    fn deserialize(_: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        unimplemented!(
            "Control instructions are not serialized and can therefore not be deserialized."
        )
    }
}
