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

    fn deserialize(_: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        unimplemented!(
            "Control instructions are not serialized and can therefore not be deserialized."
        )
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
