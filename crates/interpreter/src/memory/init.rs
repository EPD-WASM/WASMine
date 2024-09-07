use ir::instructions::MemoryInitInstruction;

use crate::{Executable, InterpreterContext, InterpreterError};

impl Executable for MemoryInitInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        unsafe {
            runtime_interface::memory_init(
                &mut ctx.exec_ctx,
                0,
                self.data_idx,
                self.s,
                self.d,
                self.n,
            )
        };

        Ok(())
    }
}
