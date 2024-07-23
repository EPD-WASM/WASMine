use crate::{Executable, InterpreterContext, InterpreterError};
use ir::instructions::MemoryCopyInstruction;

impl Executable for MemoryCopyInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        let stack_frame = ctx.stack.last_mut().unwrap();
        let size = stack_frame.vars.get(self.n).into();
        let src_offset = stack_frame.vars.get(self.s).into();
        let dst_offset = stack_frame.vars.get(self.d).into();
        unsafe {
            runtime_interface::memory_copy(ctx.exec_ctx, 0, src_offset, dst_offset, size);
        };
        Ok(())
    }
}
