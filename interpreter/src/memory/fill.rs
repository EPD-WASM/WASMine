use ir::{instructions::MemoryFillInstruction, utils::numeric_transmutes::Bit64};

use crate::{Executable, InterpreterContext, InterpreterError};

impl Executable for MemoryFillInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        let stack_frame = ctx.stack.last_mut().unwrap();

        let offset = stack_frame.vars.get(self.d).trans_u32();
        let size = stack_frame.vars.get(self.n).trans_u32();
        let value = stack_frame.vars.get(self.val) as u8;

        unsafe {
            runtime_interface::memory_fill(ctx.exec_ctx, 0, offset, size, value);
        };

        Ok(())
    }
}
