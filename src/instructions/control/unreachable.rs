use super::*;

#[derive(Debug, Clone)]
pub(crate) struct Unreachable {}

impl Instruction for Unreachable {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.finish(ControlInstruction::Unreachable);
    }

    fn deserialize(_: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        Ok(Unreachable {})
    }
}

pub(crate) fn r#unreachable(_: &mut C, _: &mut I, o: &mut O) -> ParseResult {
    o.write(Unreachable {});
    Ok(())
}
