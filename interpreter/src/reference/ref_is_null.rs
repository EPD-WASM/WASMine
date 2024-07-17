use ir::{instructions::ReferenceIsNullInstruction, utils::numeric_transmutes::Bit64};

use crate::{Executable, InterpreterContext, InterpreterError};

impl Executable for ReferenceIsNullInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        let stack_frame = ctx.stack.last_mut().unwrap();

        let in1_u64 = stack_frame.vars.get(self.in1);

        //
        let res: u64 = if in1_u64.trans_u32() == 0 {
            true as u64
        } else {
            false as u64
        };

        stack_frame.vars.set(self.out1, res);

        Ok(())
    }
}
