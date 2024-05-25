use crate::parser::parsable::ParseWithContext;

use super::*;

impl Instruction for Block {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.finish(ControlInstruction::Block(self.block_type));
    }

    fn deserialize(_: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        unimplemented!(
            "Control instructions are not serialized and can therefore not be deserialized."
        )
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
