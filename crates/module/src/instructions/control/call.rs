use super::*;
use wasm_types::FuncIdx;

#[derive(Debug, Clone)]
pub struct Call {
    pub func_idx: FuncIdx,
}

impl Instruction for Call {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.finish(ControlInstruction::Call(self.func_idx));
    }

    fn deserialize(_: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        unimplemented!(
            "Control instructions are not serialized and can therefore not be deserialized."
        )
    }
}
