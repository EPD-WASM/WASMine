use ir::instructions::WrapInstruction;

use crate::{Executable, InterpreterContext, InterpreterError};

impl Executable for WrapInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        // println!("Executing WrapInstruction: {:?}", self);
        let stack_frame = ctx.stack.last_mut().unwrap();
        let in1 = stack_frame.vars.get(self.in1).as_u64();
        let mask = u32::MAX as u64;
        stack_frame.vars.set(self.out1, (in1 & mask).into());

        Ok(())
    }
}
