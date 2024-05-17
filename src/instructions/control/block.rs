use crate::parser::parsable::ParseWithContext;

use super::*;

impl Instruction for Block {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.finish(ControlInstruction::Block(self.block_type));
    }

    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let control_instruction = i.read_terminator();
        if let ControlInstruction::Block(block_type) = control_instruction {
            Ok(Block { block_type })
        } else {
            Err(DecodingError::TypeMismatch)
        }
    }
}

pub(crate) fn block(
    ctxt: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let block_type = BlockType::parse_with_context(i, ctxt.module)?;
    o.write(Block { block_type });
    Ok(())
}
