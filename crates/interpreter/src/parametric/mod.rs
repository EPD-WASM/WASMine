use module::instructions::{DropInstruction, Instruction, SelectInstruction};
use wasm_types::{InstructionType, ParametricInstructionType, VariableInstructionType};

use crate::{Executable, InterpreterContext, InterpreterError};

mod drop;
mod select;

pub(crate) fn execute_parametric_instruction(
    ctx: &mut InterpreterContext,
    instruction_type: ParametricInstructionType,
    t: InstructionType,
) -> Result<(), InterpreterError> {
    let i = &mut ctx.stack.last_mut().unwrap().decoder;

    match instruction_type {
        ParametricInstructionType::Drop => DropInstruction::deserialize(i, t)?.execute(ctx), // TODO this is a no-op. Remove altogether?
        ParametricInstructionType::Select => SelectInstruction::deserialize(i, t)?.execute(ctx),
    }
}
