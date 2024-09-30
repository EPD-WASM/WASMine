use super::*;

#[derive(Debug, Clone)]
pub struct Unreachable {}

impl Instruction for Unreachable {
    fn deserialize(_: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        unimplemented!(
            "Control instructions are not serialized and can therefore not be deserialized."
        )
    }
}
