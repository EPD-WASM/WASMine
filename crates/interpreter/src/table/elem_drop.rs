use crate::{Executable, InterpreterContext, InterpreterError};
use ir::instructions::ElemDropInstruction;

impl Executable for ElemDropInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        unsafe {
            runtime_interface::elem_drop(ctx.exec_ctx, self.elem_idx);
        };

        Ok(())
    }
}
