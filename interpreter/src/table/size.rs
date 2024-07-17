use ir::{instructions::TableSizeInstruction, utils::numeric_transmutes::Bit32};

use crate::{Executable, InterpreterContext, InterpreterError};

impl Executable for TableSizeInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        let res = unsafe { runtime_interface::table_size(ctx.exec_ctx, self.table_idx as usize) };

        let stack_frame = ctx.stack.last_mut().unwrap();
        stack_frame.vars.set(self.out1, res.trans_u64());

        Ok(())
    }
}
