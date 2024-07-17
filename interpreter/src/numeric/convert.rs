use ir::{instructions::ConvertInstruction, structs::value::Number};
use wasm_types::NumType;

use crate::{Executable, InterpreterContext, InterpreterError};

// used for converting integer numbers to floating point numbers
impl Executable for ConvertInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        // println!("Executing ConvertInstruction: {:?}", self);
        let stack_frame = ctx.stack.last_mut().unwrap();
        let in1_u64 = stack_frame.vars.get(self.in1);

        let in1 = Number::trans_from_u64_sign(in1_u64, &self.in1_type, self.signed);

        let res: Number = match self.out1_type {
            NumType::F32 => in1.convert_to_f32(),
            NumType::F64 => in1.convert_to_f64(),
            _ => return Err(InterpreterError::InvalidType),
        };

        stack_frame.vars.set(self.out1, res.trans_to_u64());

        Ok(())
    }
}
