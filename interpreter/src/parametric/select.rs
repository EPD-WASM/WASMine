use ir::instructions::SelectInstruction;

use crate::{Executable, InterpreterContext, InterpreterError};

impl Executable for SelectInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        // println!("Executing select instruction {:?}", self);

        let stack_frame = ctx.stack.last_mut().unwrap();

        let select_var = stack_frame.vars.get(self.select_val) == 0;

        let res = stack_frame.vars.get(self.input_vals[select_var as usize]);

        stack_frame.vars.set(self.out1, res);

        Ok(())
    }
}
