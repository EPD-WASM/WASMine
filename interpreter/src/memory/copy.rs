use ir::{instructions::MemoryCopyInstruction, utils::numeric_transmutes::Bit64};

use crate::{Executable, InterpreterContext, InterpreterError};

impl Executable for MemoryCopyInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        let stack_frame = ctx.stack.last_mut().unwrap();

        let size = stack_frame.vars.get(self.n).trans_u32();
        let src_offset = stack_frame.vars.get(self.s).trans_u32();
        let dst_offset = stack_frame.vars.get(self.d).trans_u32();

        unsafe {
            runtime_interface::memory_copy(ctx.exec_ctx, 0, src_offset, dst_offset, size);
        };

        Ok(())
    }
}
