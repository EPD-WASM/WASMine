use module::instructions::LocalSetInstruction;

use crate::{Executable, InterpreterContext, InterpreterError};

impl Executable for LocalSetInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        log::trace!("{:?}", self);
        let stack_frame = ctx.stack.last_mut().unwrap();
        log::trace!("Local vars: {}", stack_frame.fn_local_vars);

        log::trace!("Getting from vars:");
        let in1_u64 = stack_frame.vars.get(self.in1);

        log::trace!("Setting local:");
        stack_frame
            .fn_local_vars
            .set(self.local_idx as usize, in1_u64);

        Ok(())
    }
}
