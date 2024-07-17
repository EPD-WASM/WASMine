use ir::instructions::ReferenceNullInstruction;

use crate::{Executable, InterpreterContext, InterpreterError};

impl Executable for ReferenceNullInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        let stack_frame = ctx.stack.last_mut().unwrap();
        stack_frame.vars.set(self.out1, 0);

        Ok(())
    }
}
