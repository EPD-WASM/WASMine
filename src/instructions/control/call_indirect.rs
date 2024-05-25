use super::*;
use wasm_types::{TableIdx, TypeIdx};

#[derive(Debug, Clone)]
pub(crate) struct CallIndirect {
    type_idx: TypeIdx,
    table_idx: TableIdx,
}

impl Instruction for CallIndirect {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.finish(ControlInstruction::CallIndirect(
            self.type_idx,
            self.table_idx,
        ));
    }

    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let control_instruction = i.read_terminator();
        if let ControlInstruction::CallIndirect(type_idx, table_idx) = control_instruction {
            Ok(CallIndirect {
                type_idx,
                table_idx,
            })
        } else {
            Err(DecodingError::TypeMismatch)
        }
    }
}

pub(crate) fn call_indirect(
    _: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let type_idx = TypeIdx::parse(i)?;
    let table_idx = TableIdx::parse(i)?;
    o.write(CallIndirect {
        type_idx,
        table_idx,
    });
    Ok(())
}
