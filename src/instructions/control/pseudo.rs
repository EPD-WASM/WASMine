use super::*;

// pseudo instructions required for basic block parsing

#[derive(Debug, Clone)]
pub(crate) struct Else {}

impl Instruction for Else {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.finish(ControlInstruction::Else);
    }

    fn deserialize(_: &mut InstructionDecoder, _t: InstructionType) -> Result<Self, DecodingError> {
        Ok(Else {})
    }
}

pub(crate) fn r#else(
    _: &mut Context,
    _: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    o.write(Else {});
    Ok(())
}

#[derive(Debug, Clone)]
pub(crate) struct End {}

impl Instruction for End {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.finish(ControlInstruction::End);
    }
}

pub(crate) fn end(
    _: &mut Context,
    _: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    o.write(End {});
    Ok(())
}
