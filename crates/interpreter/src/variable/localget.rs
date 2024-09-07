use ir::instructions::LocalGetInstruction;

use crate::{util, Executable, InterpreterContext, InterpreterError};

impl Executable for LocalGetInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        // println!("Executing LocalGetInstruction: {:?}", self);
        let stack_frame = ctx.stack.last_mut().unwrap();

        let value = stack_frame.fn_local_vars.get(self.local_idx);

        stack_frame.vars.set(self.out1, value);

        Ok(())
    }
}
