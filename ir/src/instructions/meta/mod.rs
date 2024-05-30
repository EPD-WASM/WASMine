use super::*;
use crate::{basic_block::BasicBlockID, DecodingError, InstructionDecoder, InstructionEncoder};
use wasm_types::{InstructionType, MetaInstructionType};

use super::{Instruction, VariableID};

#[derive(Debug, Clone)]
pub struct PhiNode {
    pub inputs: Vec<(BasicBlockID, VariableID)>,
    pub out: VariableID,
}

impl Instruction for PhiNode {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.write_instruction_type(InstructionType::Meta(MetaInstructionType::PhiNode));
        o.write_immediate(self.inputs.len() as u64);
        for (bb, var) in self.inputs {
            o.write_variable(var);
            // we fake the basicblockid as a variable (don't judge)
            o.write_variable(bb);
        }
        o.write_variable(self.out);
    }

    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let len = i.read_immediate()?;
        let mut inputs = Vec::with_capacity(len as usize);
        for _ in 0..len {
            inputs.push((i.read_variable()?, i.read_variable()?));
        }
        Ok(PhiNode {
            inputs,
            out: i.read_variable()?,
        })
    }
}

impl Display for PhiNode {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "%{} = phi ", self.out)?;
        let inputs = self
            .inputs
            .iter()
            .map(|(bb, var)| format!("[ %{}, bb{} ]", var, bb))
            .collect::<Vec<_>>()
            .join(", ");
        write!(f, "{}", inputs)
    }
}
