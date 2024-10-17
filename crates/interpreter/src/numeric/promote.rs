use crate::{Executable, InterpreterContext, InterpreterError};
use module::{instructions::PromoteInstruction, utils::numeric_transmutes::Bit32};

impl Executable for PromoteInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        let stack_frame = ctx.stack.last_mut().unwrap();
        let in1: f32 = stack_frame.vars.get(self.in1).as_f32().trans_f32();
        let promoted: f64 = in1 as f64;

        stack_frame.vars.set(self.out1, promoted.into());

        Ok(())
    }
}
