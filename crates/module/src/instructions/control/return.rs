use super::*;

#[derive(Debug, Clone)]
pub struct Return {}

impl Instruction for Return {
    fn deserialize(_: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        unimplemented!(
            "Control instructions are not serialized and can therefore not be deserialized."
        )
    }
}
