use module::{
    instructions::ConvertInstruction,
    objects::value::{Number, Value},
};
use wasm_types::{NumType, ValType};

use crate::{Executable, InterpreterContext, InterpreterError};

// used for converting integer numbers to floating point numbers
impl Executable for ConvertInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        let stack_frame = ctx.stack.last_mut().unwrap();
        let in1 = stack_frame
            .vars
            .get_number_signed(self.in1, self.in1_type, self.signed);

        let res: Number = match self.out1_type {
            NumType::F32 => in1.convert_to_f32(),
            NumType::F64 => in1.convert_to_f64(),
            _ => return Err(InterpreterError::InvalidType),
        };
        
        stack_frame.vars.set(self.out1, Value::Number(res).into());
        Ok(())
    }
}
