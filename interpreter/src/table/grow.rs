use crate::{Executable, InterpreterContext, InterpreterError};
use ir::instructions::TableGrowInstruction;

impl Executable for TableGrowInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        let stack_frame = ctx.stack.last_mut().unwrap();

        let grow_by = stack_frame.vars.get(self.size).into();
        let value_to_fill = stack_frame.vars.get(self.value_to_fill);

        let res = unsafe {
            runtime_interface::table_grow(
                ctx.exec_ctx,
                self.table_idx,
                grow_by,
                value_to_fill.as_u64(),
            )
        };

        stack_frame.vars.set(self.out1, res.into());

        Ok(())
    }
}
