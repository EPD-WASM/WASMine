use super::*;

// pseudo instructions required for basic block parsing

#[derive(Debug, Clone)]
pub struct Else {}

impl Instruction for Else {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.finish(ControlInstruction::Else);
    }

    fn deserialize(_: &mut InstructionDecoder, _t: InstructionType) -> Result<Self, DecodingError> {
        unimplemented!(
            "Control instructions are not serialized and can therefore not be deserialized."
        )
    }
}

#[derive(Debug, Clone)]
pub struct End {}

impl Instruction for End {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.finish(ControlInstruction::End);
    }
}
