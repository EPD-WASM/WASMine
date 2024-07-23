use ir::instructions::TableGetInstruction;

use crate::{Executable, InterpreterContext, InterpreterError};

impl Executable for TableGetInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        let val = unsafe {
            runtime_interface::table_get(ctx.exec_ctx, self.table_idx as usize, self.idx)
        };

        let stack_frame = ctx.stack.last_mut().unwrap();

        stack_frame.vars.set(self.out1, val.into());

        Ok(())
    }
}
