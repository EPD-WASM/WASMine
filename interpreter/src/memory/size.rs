use crate::{Executable, InterpreterContext, InterpreterError};
use ir::{instructions::MemorySizeInstruction, utils::numeric_transmutes::Bit32};

impl Executable for MemorySizeInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        // hehe
        let res = unsafe { runtime_interface::memory_grow(&mut ctx.exec_ctx, 0, 0) };

        let stack_frame = ctx.stack.last_mut().unwrap();
        stack_frame.vars.set(self.out1, res.trans_u64());

        Ok(())
    }
}
