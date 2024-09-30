use super::*;
use wasm_types::LabelIdx;

#[derive(Debug, Clone)]
pub struct Br {
    pub label_idx: LabelIdx,
}

impl Instruction for Br {
    fn deserialize(_: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        unimplemented!(
            "Control instructions are not serialized and can therefore not be deserialized."
        )
    }
}
