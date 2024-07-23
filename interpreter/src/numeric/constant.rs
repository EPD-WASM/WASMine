use crate::{Executable, InterpreterContext, InterpreterError};
use ir::instructions::Constant;

impl Executable for Constant {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        let stack_frame = ctx.stack.last_mut().unwrap();
        stack_frame.vars.set(self.out1, self.imm);
        Ok(())
    }
}
