use super::*;
use wasm_types::{TableIdx, TypeIdx};

#[derive(Debug, Clone)]
pub struct CallIndirect {
    pub type_idx: TypeIdx,
    pub table_idx: TableIdx,
}

impl Instruction for CallIndirect {
    fn deserialize(_: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        unimplemented!(
            "Control instructions are not serialized and can therefore not be deserialized."
        )
    }
}
