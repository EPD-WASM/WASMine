use super::*;

#[derive(Debug, Clone)]
pub(crate) struct Return {}

impl Instruction for Return {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.finish(ControlInstruction::Return);
    }

    fn deserialize(_: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        Ok(Return {})
    }
}

pub(crate) fn r#return(
    _: &mut Context,
    _: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    o.write(Return {});
    Ok(())
}
