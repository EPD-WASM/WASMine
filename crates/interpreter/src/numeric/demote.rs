use crate::{Executable, InterpreterContext, InterpreterError};
use module::{instructions::DemoteInstruction, utils::numeric_transmutes::Bit64};

impl Executable for DemoteInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        let stack_frame = ctx.stack.last_mut().unwrap();
        let in1 = stack_frame.vars.get(self.in1).as_f64().trans_f64();
        let demoted = in1 as f32;

        stack_frame.vars.set(self.out1, demoted.into());

        Ok(())
    }
}
