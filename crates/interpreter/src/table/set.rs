use module::instructions::TableSetInstruction;

use crate::{Executable, InterpreterContext, InterpreterError};

impl Executable for TableSetInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        let stack_frame = ctx.stack.last_mut().unwrap();
        let val = stack_frame.vars.get(self.in1);

        // unsafe {
        //     runtime_interface::table_set(
        //         ctx.exec_ctx,
        //         self.table_idx as usize,
        //         val.as_u64(),
        //         self.idx,
        //     );
        // };

        Ok(())
    }
}
