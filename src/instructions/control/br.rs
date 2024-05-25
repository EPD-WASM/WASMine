use super::*;
use wasm_types::LabelIdx;

#[derive(Debug, Clone)]
pub(crate) struct Br {
    label_idx: LabelIdx,
}

impl Instruction for Br {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.finish(ControlInstruction::Br(self.label_idx));
    }

    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let control_instruction = i.read_terminator();
        if let ControlInstruction::Br(label_idx) = control_instruction {
            Ok(Br { label_idx })
        } else {
            Err(DecodingError::TypeMismatch)
        }
    }
}

pub(crate) fn br(
    _: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let label_idx = LabelIdx::parse(i)?;
    o.write(Br { label_idx });
    Ok(())
}
