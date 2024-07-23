use crate::{Executable, InterpreterContext, InterpreterError};
use ir::instructions::TableInitInstruction;

impl Executable for TableInitInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        let stack_frame = ctx.stack.last_mut().unwrap();

        let src_offset = stack_frame.vars.get(self.s).into();
        let dst_offset = stack_frame.vars.get(self.d).into();
        let length = stack_frame.vars.get(self.n).into();

        unsafe {
            runtime_interface::table_init(
                ctx.exec_ctx,
                self.table_idx,
                self.elem_idx,
                src_offset,
                dst_offset,
                length,
            )
        };

        Ok(())
    }
}
