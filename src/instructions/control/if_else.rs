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

    fn deserialize(_: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        unimplemented!(
            "Control instructions are not serialized and can therefore not be deserialized."
        )
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
