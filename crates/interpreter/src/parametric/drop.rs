use module::instructions::DropInstruction;

use crate::{Executable, InterpreterContext, InterpreterError};

impl Executable for DropInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        // this is a no-op
        Ok(())
    }
}
