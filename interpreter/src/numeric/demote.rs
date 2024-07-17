use ir::{
    instructions::DemoteInstruction,
    utils::numeric_transmutes::{Bit32, Bit64},
};

use crate::{Executable, InterpreterContext, InterpreterError};

impl Executable for DemoteInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        let stack_frame = ctx.stack.last_mut().unwrap();
        let in1 = stack_frame.vars.get(self.in1).trans_f64();
        let demoted = in1 as f32;

        stack_frame.vars.set(self.out1, demoted.trans_u64());

        Ok(())
    }
}
