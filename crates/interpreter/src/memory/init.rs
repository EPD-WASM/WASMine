use module::instructions::MemoryInitInstruction;

use crate::{Executable, InterpreterContext, InterpreterError};

impl Executable for MemoryInitInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        let stack_frame = ctx.stack.last_mut().unwrap();
        let n = stack_frame.vars.get(self.n).as_u32();
        let s = stack_frame.vars.get(self.s).as_u32();
        let d = stack_frame.vars.get(self.d).as_u32();

        unsafe { runtime_interface::memory_init(&mut ctx.exec_ctx, 0, self.data_idx, s, d, n) };

        Ok(())
    }
}
