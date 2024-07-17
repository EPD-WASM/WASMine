use ir::{
    instructions::TableGrowInstruction,
    utils::numeric_transmutes::{Bit32, Bit64},
};

use crate::{Executable, InterpreterContext, InterpreterError};

impl Executable for TableGrowInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        let stack_frame = ctx.stack.last_mut().unwrap();

        let grow_by = stack_frame.vars.get(self.size).trans_u32();
        let value_to_fill = stack_frame.vars.get(self.value_to_fill);

        let res = unsafe {
            runtime_interface::table_grow(ctx.exec_ctx, self.table_idx, grow_by, value_to_fill)
        };

        stack_frame.vars.set(self.out1, res.trans_u64());

        Ok(())
    }
}
