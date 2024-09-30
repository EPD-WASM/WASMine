use module::instructions::LocalTeeInstruction;

use crate::{util, Executable, InterpreterContext, InterpreterError};

impl Executable for LocalTeeInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        // println!("Executing LocalSetInstruction: {:?}", self);
        // let stack_frame = ctx.stack.last_mut().unwrap();

        // let in1_u64 = stack_frame.vars.get(self.in1);

        // stack_frame.fn_local_vars.set(self.local_idx, in1_u64);
        // stack_frame.vars.set(self.out1, in1_u64);

        Ok(())
    }
}
