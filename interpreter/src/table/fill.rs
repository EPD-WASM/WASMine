use ir::{instructions::TableFillInstruction, utils::numeric_transmutes::Bit64};

use crate::{Executable, InterpreterContext, InterpreterError};

impl Executable for TableFillInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        let stack_frame = ctx.stack.last_mut().unwrap();
        let length = stack_frame.vars.get(self.n).trans_u32();
        let val = stack_frame.vars.get(self.ref_value);
        let start = stack_frame.vars.get(self.i).trans_u32();

        unsafe {
            runtime_interface::table_fill(
                ctx.exec_ctx,
                self.table_idx as usize,
                start,
                length,
                val,
            );
        };

        Ok(())
    }
}
