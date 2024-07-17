use ir::{
    instructions::ReferenceFunctionInstruction,
    structs::value::{Reference, Value},
};

use crate::{Executable, InterpreterContext, InterpreterError};

impl Executable for ReferenceFunctionInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        let stack_frame = ctx.stack.last_mut().unwrap();
        let reference = Reference::Function(self.func_idx);
        let val = Value::Reference(reference);

        stack_frame.vars.set(self.out1, val.trans_to_u64());

        Ok(())
    }
}
