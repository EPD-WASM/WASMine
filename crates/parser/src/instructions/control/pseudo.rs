use super::*;

// pseudo instructions required for basic block parsing

pub(crate) fn r#else(
    _: &mut Context,
    _: &mut WasmBinaryReader,
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
    _: &mut WasmBinaryReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    o.write(End {});
    Ok(())
}
