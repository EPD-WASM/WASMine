use module::instructions::{
    GlobalGetInstruction, GlobalSetInstruction, Instruction, LocalGetInstruction,
    LocalSetInstruction, LocalTeeInstruction,
};
use wasm_types::{
    InstructionType,
    VariableInstructionType::{self, *},
};

use crate::{Executable, InterpreterContext, InterpreterError};

mod globalget;
mod globalset;
mod localget;
mod localset;
mod localtee;

pub(crate) fn execute_variable_instruction(
    ctx: &mut InterpreterContext,
    instruction_type: VariableInstructionType,
    t: InstructionType,
) -> Result<(), InterpreterError> {
    let i = &mut ctx.stack.last_mut().unwrap().decoder;
    match instruction_type {
        LocalGet => LocalGetInstruction::deserialize(i, t)?.execute(ctx),
        LocalSet => LocalSetInstruction::deserialize(i, t)?.execute(ctx),
        LocalTee => LocalTeeInstruction::deserialize(i, t)?.execute(ctx),
        GlobalGet => GlobalGetInstruction::deserialize(i, t)?.execute(ctx),
        GlobalSet => GlobalSetInstruction::deserialize(i, t)?.execute(ctx),
    }
}
