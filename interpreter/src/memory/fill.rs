use crate::{Executable, InterpreterContext, InterpreterError};
use ir::instructions::MemoryFillInstruction;

impl Executable for MemoryFillInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        let stack_frame = ctx.stack.last_mut().unwrap();
        let offset = stack_frame.vars.get(self.d).into();
        let size = stack_frame.vars.get(self.n).into();
        let value = stack_frame.vars.get(self.val).as_u32() as u8;
        unsafe {
            runtime_interface::memory_fill(ctx.exec_ctx, 0, offset, size, value);
        };
        Ok(())
    }
}
