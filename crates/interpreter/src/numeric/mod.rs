use super::{Executable, InterpreterContext, InterpreterError};
use module::instructions::{
    Constant, ConvertInstruction, DemoteInstruction, ExtendBitsInstruction, ExtendTypeInstruction,
    FBinaryInstruction, FRelationalInstruction, FUnaryInstruction, IBinaryInstruction,
    IRelationalInstruction, ITestInstruction, IUnaryInstruction, Instruction, PromoteInstruction,
    ReinterpretInstruction, TruncInstruction, TruncSaturationInstruction, WrapInstruction,
};
use wasm_types::{ConversionOp, InstructionType, NumericInstructionCategory};

mod constant;
mod convert;
mod demote;
mod extendbits;
mod extendtype;
mod fbinary;
mod frelational;
mod funary;
mod ibinary;
mod irelational;
mod itest;
mod iunary;
mod promote;
mod reinterpret;
mod trunc;
mod truncsat;
mod wrap;

pub(super) fn execute_numeric_instruction(
    ctx: &mut InterpreterContext,
    category: NumericInstructionCategory,
    t: InstructionType,
) -> Result<(), InterpreterError> {
    let i = &mut ctx.stack.last_mut().unwrap().decoder;
    // TODO fix the formatting rules for this
    match category {
        NumericInstructionCategory::IUnary(_) => IUnaryInstruction::deserialize(i, t)?.execute(ctx),
        NumericInstructionCategory::IBinary(_) => {
            IBinaryInstruction::deserialize(i, t)?.execute(ctx)
        }
        NumericInstructionCategory::Constant => Constant::deserialize(i, t)?.execute(ctx),
        NumericInstructionCategory::FUnary(_) => FUnaryInstruction::deserialize(i, t)?.execute(ctx),
        NumericInstructionCategory::FBinary(_) => {
            FBinaryInstruction::deserialize(i, t)?.execute(ctx)
        }
        NumericInstructionCategory::IRelational(_) => {
            IRelationalInstruction::deserialize(i, t)?.execute(ctx)
        }
        NumericInstructionCategory::ITest(_) => ITestInstruction::deserialize(i, t)?.execute(ctx),

        NumericInstructionCategory::FRelational(_) => {
            FRelationalInstruction::deserialize(i, t)?.execute(ctx)
        }
        NumericInstructionCategory::Conversion(op) => match op {
            ConversionOp::Wrap => WrapInstruction::deserialize(i, t)?.execute(ctx),
            ConversionOp::ExtendBits => ExtendBitsInstruction::deserialize(i, t)?.execute(ctx),
            ConversionOp::ExtendType => ExtendTypeInstruction::deserialize(i, t)?.execute(ctx),
            ConversionOp::Trunc => TruncInstruction::deserialize(i, t)?.execute(ctx),
            ConversionOp::TruncSat => TruncSaturationInstruction::deserialize(i, t)?.execute(ctx),
            ConversionOp::Demote => DemoteInstruction::deserialize(i, t)?.execute(ctx),
            ConversionOp::Promote => PromoteInstruction::deserialize(i, t)?.execute(ctx),
            ConversionOp::Convert => ConvertInstruction::deserialize(i, t)?.execute(ctx),
            ConversionOp::Reinterpret => ReinterpretInstruction::deserialize(i, t)?.execute(ctx),
        },
    }
}

// impl Executable for ...
