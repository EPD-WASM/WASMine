use module::instructions::TableGetInstruction;

use crate::{Executable, InterpreterContext, InterpreterError};

impl Executable for TableGetInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        let stack_frame = ctx.stack.last_mut().unwrap();

        let idx = stack_frame.vars.get(self.idx).as_u32();
        let val =
            unsafe { runtime_interface::table_get(ctx.exec_ctx, self.table_idx as usize, idx) };

        stack_frame.vars.set(self.out1, val.into());

        Ok(())
    }
}
