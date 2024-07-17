use ir::{instructions::TableInitInstruction, utils::numeric_transmutes::Bit64};

use crate::{Executable, InterpreterContext, InterpreterError};

impl Executable for TableInitInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        let stack_frame = ctx.stack.last_mut().unwrap();

        let src_offset = stack_frame.vars.get(self.s).trans_u32();
        let dst_offset = stack_frame.vars.get(self.d).trans_u32();
        let length = stack_frame.vars.get(self.n).trans_u32();

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
