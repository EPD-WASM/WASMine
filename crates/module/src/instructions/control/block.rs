use super::*;

#[derive(Debug, Clone)]
pub struct Block {
    pub block_type: BlockType,
}

impl Instruction for Block {
    fn deserialize(_: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        unimplemented!(
            "Control instructions are not serialized and can therefore not be deserialized."
        )
    }
}
