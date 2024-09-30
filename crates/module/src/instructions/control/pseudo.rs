use super::*;

// pseudo instructions required for basic block parsing

#[derive(Debug, Clone)]
pub struct Else {}

impl Instruction for Else {
    fn deserialize(_: &mut InstructionDecoder, _t: InstructionType) -> Result<Self, DecodingError> {
        unimplemented!(
            "Control instructions are not serialized and can therefore not be deserialized."
        )
    }
}

#[derive(Debug, Clone)]
pub struct End {}

impl Instruction for End {}
