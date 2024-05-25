use super::*;
use wasm_types::LabelIdx;

#[derive(Debug, Clone)]
pub(crate) struct BrTable {
    label_indices: Vec<LabelIdx>,
    default_label_idx: LabelIdx,
}

impl Instruction for BrTable {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.finish(ControlInstruction::BrTable(
            self.default_label_idx,
            self.label_indices,
        ));
    }

    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let control_instruction = i.read_terminator();
        if let ControlInstruction::BrTable(default_label_idx, label_indices) = control_instruction {
            Ok(BrTable {
                default_label_idx,
                label_indices,
            })
        } else {
            Err(DecodingError::TypeMismatch)
        }
    }
}

pub(crate) fn br_table(
    _: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let num_labels = i.read_leb128::<u32>()?;
    let label_indices = (0..num_labels)
        .map(|_| LabelIdx::parse(i))
        .collect::<Result<Vec<LabelIdx>, ParserError>>()?;
    let default_label_idx = LabelIdx::parse(i)?;
    o.write(BrTable {
        label_indices,
        default_label_idx,
    });
    Ok(())
}
