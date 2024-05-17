use crate::parser::parsable::ParseWithContext;

use super::*;

#[derive(Debug, Clone)]
pub(crate) struct Loop {
    block_type: BlockType,
}

impl Instruction for Loop {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.finish(ControlInstruction::Loop(self.block_type));
    }

    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let control_instruction = i.read_terminator();
        if let ControlInstruction::Loop(block_type) = control_instruction {
            Ok(Loop { block_type })
        } else {
            Err(DecodingError::TypeMismatch)
        }
    }
}

pub(crate) fn r#loop(
    ctxt: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let block_type = BlockType::parse_with_context(i, ctxt.module)?;
    o.write(Loop { block_type });
    Ok(())
}
