use ir::instructions::{
    ElemDropInstruction, Instruction, TableCopyInstruction, TableFillInstruction,
    TableGetInstruction, TableGrowInstruction, TableInitInstruction, TableSetInstruction,
    TableSizeInstruction,
};
use wasm_types::{InstructionType, MemoryInstructionCategory, TableInstructionCategory};

use crate::{Executable, InterpreterContext, InterpreterError};

mod copy;
mod elem_drop;
mod fill;
mod get;
mod grow;
mod init;
mod set;
mod size;

pub(crate) fn execute_table_instruction(
    ctx: &mut InterpreterContext,
    instruction_category: TableInstructionCategory,
    t: InstructionType,
) -> Result<(), InterpreterError> {
    let i = &mut ctx.stack.last_mut().unwrap().decoder;

    match instruction_category {
        TableInstructionCategory::Get => TableGetInstruction::deserialize(i, t)?.execute(ctx),
        TableInstructionCategory::Set => TableSetInstruction::deserialize(i, t)?.execute(ctx),
        TableInstructionCategory::Size => TableSizeInstruction::deserialize(i, t)?.execute(ctx),
        TableInstructionCategory::Grow => TableGrowInstruction::deserialize(i, t)?.execute(ctx),
        TableInstructionCategory::Fill => TableFillInstruction::deserialize(i, t)?.execute(ctx),
        TableInstructionCategory::Copy => TableCopyInstruction::deserialize(i, t)?.execute(ctx),
        TableInstructionCategory::Init => TableInitInstruction::deserialize(i, t)?.execute(ctx),
        TableInstructionCategory::Drop => ElemDropInstruction::deserialize(i, t)?.execute(ctx),
    }
}
