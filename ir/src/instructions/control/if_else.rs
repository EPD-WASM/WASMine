use super::*;

#[derive(Debug, Clone)]
pub struct IfElse {
    pub block_type: BlockType,
}

impl Instruction for IfElse {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.finish(ControlInstruction::IfElse(self.block_type));
    }

    fn deserialize(_: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        unimplemented!(
            "Control instructions are not serialized and can therefore not be deserialized."
        )
    }
}
