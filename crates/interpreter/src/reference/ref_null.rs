use crate::{Executable, InterpreterContext, InterpreterError};
use module::{
    instructions::ReferenceNullInstruction,
    objects::value::{Reference, Value},
};

impl Executable for ReferenceNullInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        let stack_frame = ctx.stack.last_mut().unwrap();
        stack_frame
            .vars
            .set(self.out1, Value::Reference(Reference::Null).into());

        Ok(())
    }
}
