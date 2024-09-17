use module::instructions::{
    Instruction, LoadInstruction, MemoryCopyInstruction, MemoryFillInstruction,
    MemoryGrowInstruction, MemoryInitInstruction, MemorySizeInstruction, StoreInstruction,
};
use wasm_types::{InstructionType, MemoryInstructionCategory, MemoryOp};

use crate::{Executable, InterpreterContext, InterpreterError};

mod copy;
mod fill;
mod grow;
mod init;
mod load;
mod size;
mod store;

pub(crate) fn execute_memory_instruction(
    ctx: &mut InterpreterContext,
    instruction_category: MemoryInstructionCategory,
    t: InstructionType,
) -> Result<(), InterpreterError> {
    let i = &mut ctx.stack.last_mut().unwrap().decoder;

    match instruction_category {
        MemoryInstructionCategory::Load(_) => LoadInstruction::deserialize(i, t)?.execute(ctx),
        MemoryInstructionCategory::Store(_) => StoreInstruction::deserialize(i, t)?.execute(ctx),
        MemoryInstructionCategory::Memory(MemoryOp::Copy) => {
            MemoryCopyInstruction::deserialize(i, t)?.execute(ctx)
        }
        MemoryInstructionCategory::Memory(MemoryOp::Fill) => {
            MemoryFillInstruction::deserialize(i, t)?.execute(ctx)
        }
        MemoryInstructionCategory::Memory(MemoryOp::Size) => {
            MemorySizeInstruction::deserialize(i, t)?.execute(ctx)
        }
        MemoryInstructionCategory::Memory(MemoryOp::Grow) => {
            MemoryGrowInstruction::deserialize(i, t)?.execute(ctx)
        }
        MemoryInstructionCategory::Memory(MemoryOp::Init) => {
            MemoryInitInstruction::deserialize(i, t)?.execute(ctx)
        }
        MemoryInstructionCategory::Memory(MemoryOp::Drop) => todo!(),
    }
}
