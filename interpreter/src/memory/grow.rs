use core::slice;

use ir::{
    instructions::MemoryGrowInstruction,
    utils::numeric_transmutes::{Bit32, Bit64},
};

use crate::{Executable, InterpreterContext, InterpreterError};

impl Executable for MemoryGrowInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        let stack_frame = ctx.stack.last_mut().unwrap();

        let grow_by = stack_frame.vars.get(self.in1).trans_u32();

        let res = unsafe { runtime_interface::memory_grow(ctx.exec_ctx, 0, grow_by) };

        stack_frame.vars.set(self.out1, res.trans_u64());

        Ok(())
    }
}
