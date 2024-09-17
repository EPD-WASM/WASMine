use crate::{Executable, InterpreterContext, InterpreterError};
use module::instructions::TableFillInstruction;

impl Executable for TableFillInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        let stack_frame = ctx.stack.last_mut().unwrap();
        let length = stack_frame.vars.get(self.n).into();
        let val = stack_frame.vars.get(self.ref_value);
        let start = stack_frame.vars.get(self.i).into();

        unsafe {
            runtime_interface::table_fill(
                ctx.exec_ctx,
                self.table_idx as usize,
                start,
                length,
                val.as_u64(),
            );
        };

        Ok(())
    }
}
