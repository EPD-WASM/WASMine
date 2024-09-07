use super::*;
use crate::{basic_block::BasicBlockID, DecodingError, InstructionDecoder, InstructionEncoder};
use bitcode::{Decode, Encode};
use wasm_types::InstructionType;

use super::{Instruction, VariableID};

#[derive(Debug, Clone, Decode, Encode)]
pub struct PhiNode {
    pub inputs: Vec<(BasicBlockID, VariableID)>,
    pub out: VariableID,
    pub r#type: ValType,
}

impl Instruction for PhiNode {
    fn serialize(self, _: &mut InstructionEncoder) {
        unimplemented!("Phis are not serialized.")
    }

    fn deserialize(_: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        unimplemented!("Phis are not deserialized.")
    }
}

impl Display for PhiNode {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "%{} = phi ", self.out)?;
        let inputs = self
            .inputs
            .iter()
            .map(|(bb, var)| format!("[ %{var}, bb{bb} ]"))
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "{inputs}")
    }
}
