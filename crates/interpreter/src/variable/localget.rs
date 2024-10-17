use module::instructions::LocalGetInstruction;

use crate::{Executable, InterpreterContext, InterpreterError};

impl Executable for LocalGetInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        log::trace!("{:?}", self);
        let stack_frame = ctx.stack.last_mut().unwrap();

        log::trace!("Local vars: {}", stack_frame.fn_local_vars);

        log::trace!("Getting local:");
        let value = stack_frame.fn_local_vars.get(self.local_idx as usize);

        log::trace!("Setting var:");
        stack_frame.vars.set(self.out1, value);

        Ok(())
    }
}
