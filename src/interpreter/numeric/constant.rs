use crate::{
    instructions::Constant,
    interpreter::{Executable, InterpreterContext, InterpreterError},
};

impl Executable for Constant {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        let stack_frame = ctx.stack.last_mut().unwrap();

        stack_frame.vars.set(self.out1, self.imm);

        Ok(())
    }
}
