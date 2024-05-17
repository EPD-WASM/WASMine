use crate::parser::parsable::ParseWithContext;

use super::*;

#[derive(Debug, Clone)]
pub(crate) struct IfElse {
    block_type: BlockType,
}

impl Instruction for IfElse {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.finish(ControlInstruction::IfElse(self.block_type));
    }

    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let control_instruction = i.read_terminator();
        if let ControlInstruction::IfElse(block_type) = control_instruction {
            Ok(IfElse { block_type })
        } else {
            Err(DecodingError::TypeMismatch)
        }
    }
}

pub(crate) fn if_else(
    ctxt: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let block_type = BlockType::parse_with_context(i, ctxt.module)?;
    o.write(IfElse { block_type });
    Ok(())
}
