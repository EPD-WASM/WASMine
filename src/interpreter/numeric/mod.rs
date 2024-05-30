use crate::{
    instructions::{
        Constant, FBinaryInstruction, FUnaryInstruction, IBinaryInstruction, IUnaryInstruction,
        Instruction,
    },
};
use wasm_types::{InstructionType, NumericInstructionCategory};

use super::{Executable, InterpreterContext, InterpreterError};

mod constant;
mod fbinary;
mod funary;
mod ibinary;
mod iunary;

pub(super) fn execute_numeric_instruction(
    ctx: &mut InterpreterContext,
    category: NumericInstructionCategory,
    // i: &mut InstructionDecoder,
    t: InstructionType,
) -> Result<(), InterpreterError> {
    let i = &mut ctx.stack.last_mut().unwrap().decoder;
    match category {
        NumericInstructionCategory::IUnary(_) => IUnaryInstruction::deserialize(i, t)
            .map_err(InterpreterError::DecodingError)?
            .execute(ctx),
        NumericInstructionCategory::IBinary(_) => IBinaryInstruction::deserialize(i, t)
            .map_err(InterpreterError::DecodingError)?
            .execute(ctx),
        NumericInstructionCategory::Constant => Constant::deserialize(i, t)
            .map_err(InterpreterError::DecodingError)?
            .execute(ctx),
        NumericInstructionCategory::FUnary(_) => FUnaryInstruction::deserialize(i, t)
            .map_err(InterpreterError::DecodingError)?
            .execute(ctx),
        NumericInstructionCategory::FBinary(_) => FBinaryInstruction::deserialize(i, t)
            .map_err(InterpreterError::DecodingError)?
            .execute(ctx),
        NumericInstructionCategory::ITest(_) => todo!(),
        NumericInstructionCategory::IRelational(_) => todo!(),
        NumericInstructionCategory::Conversion(_) => todo!(),
        NumericInstructionCategory::FRelational(_) => todo!(),
    }
}

// impl Executable for ...
