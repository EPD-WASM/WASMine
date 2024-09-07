use crate::{Executable, InterpreterContext, InterpreterError};
use ir::instructions::PromoteInstruction;

impl Executable for PromoteInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        let stack_frame = ctx.stack.last_mut().unwrap();
        let in1 = stack_frame.vars.get(self.in1).as_f32();
        let promoted = in1 as f64;

        stack_frame.vars.set(self.out1, promoted.into());

        Ok(())
    }
}
