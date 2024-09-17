use crate::{Executable, InterpreterContext, InterpreterError};
use module::instructions::MemoryGrowInstruction;

impl Executable for MemoryGrowInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        let stack_frame = ctx.stack.last_mut().unwrap();
        let grow_by = stack_frame.vars.get(self.in1).into();
        let res = unsafe { runtime_interface::memory_grow(ctx.exec_ctx, 0, grow_by) };
        stack_frame.vars.set(self.out1, res.into());

        Ok(())
    }
}
