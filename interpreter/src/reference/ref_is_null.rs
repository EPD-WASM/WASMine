use crate::{Executable, InterpreterContext, InterpreterError};
use ir::instructions::ReferenceIsNullInstruction;

impl Executable for ReferenceIsNullInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        let stack_frame = ctx.stack.last_mut().unwrap();

        let in1_u64 = stack_frame.vars.get(self.in1);

        //
        let res: u64 = if in1_u64.as_u32() == 0 {
            true as u64
        } else {
            false as u64
        };

        stack_frame.vars.set(self.out1, res.into());

        Ok(())
    }
}
