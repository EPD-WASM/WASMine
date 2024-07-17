use ir::instructions::{
    Instruction, ReferenceFunctionInstruction, ReferenceIsNullInstruction, ReferenceNullInstruction,
};
use wasm_types::{InstructionType, ReferenceInstructionType};

use crate::{Executable, InterpreterContext, InterpreterError};

mod ref_func;
mod ref_is_null;
mod ref_null;

pub(crate) fn execute_reference_instruction(
    ctx: &mut InterpreterContext,
    instruction_category: ReferenceInstructionType,
    t: InstructionType,
) -> Result<(), InterpreterError> {
    let i = &mut ctx.stack.last_mut().unwrap().decoder;

    match instruction_category {
        ReferenceInstructionType::RefNull => {
            ReferenceNullInstruction::deserialize(i, t)?.execute(ctx)?
        }
        ReferenceInstructionType::RefIsNull => {
            ReferenceIsNullInstruction::deserialize(i, t)?.execute(ctx)?
        }
        ReferenceInstructionType::RefFunc => {
            ReferenceFunctionInstruction::deserialize(i, t)?.execute(ctx)?
        }
    }

    Ok(())
}
