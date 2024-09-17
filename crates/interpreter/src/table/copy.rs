use crate::{Executable, InterpreterContext, InterpreterError};
use module::instructions::TableCopyInstruction;

impl Executable for TableCopyInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        let stack_frame = ctx.stack.last_mut().unwrap();
        let len = stack_frame.vars.get(self.n).into();
        let src_start = stack_frame.vars.get(self.s).into();
        let dst_start = stack_frame.vars.get(self.d).into();

        unsafe {
            runtime_interface::table_copy(
                ctx.exec_ctx,
                self.table_idx_x,
                self.table_idx_y,
                src_start,
                dst_start,
                len,
            )
        }

        Ok(())
    }
}
