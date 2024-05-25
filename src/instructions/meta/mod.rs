use super::*;
use crate::{
    ir::{basic_block::BasicBlockID, DecodingError, InstructionDecoder, InstructionEncoder},
    parser::Context,
};
use wasm_types::{InstructionType, MetaInstructionType, ValType};

#[derive(Debug, Clone)]
pub(crate) struct PhiNode {
    pub(crate) inputs: Vec<(BasicBlockID, VariableID)>,
    pub(crate) out: VariableID,
}

impl PhiNode {
    // we trust the caller to verify that all input variables are of the same type
    pub(crate) fn new(
        inputs: Vec<(BasicBlockID, VariableID)>,
        var_type: ValType,
        ctxt: &mut Context,
    ) -> Self {
        let out = ctxt.create_var(var_type);
        let res = PhiNode {
            inputs,
            out: out.id,
        };
        ctxt.push_var(out);
        res
    }
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
