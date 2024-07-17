use ir::{instructions::ReinterpretInstruction, utils::numeric_transmutes::Bit64};

use crate::{Executable, InterpreterContext, InterpreterError};

impl Executable for ReinterpretInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        // println!("Executing ReinterpretInstruction: {:?}", self);
        let stack_frame = ctx.stack.last_mut().unwrap();

        let in1 = stack_frame.vars.get(self.in1);
        stack_frame.vars.set(self.out1, in1);

        Ok(())
    }
}
