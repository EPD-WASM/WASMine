use module::instructions::DataDropInstruction;

use crate::{Executable, InterpreterContext, InterpreterError};

impl Executable for DataDropInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        unsafe {
            runtime_interface::data_drop(ctx.exec_ctx, self.data_idx);
        }

        Ok(())
    }
}
