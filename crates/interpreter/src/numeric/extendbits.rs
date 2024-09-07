use ir::{
    instructions::ExtendBitsInstruction,
    structs::value::{Number, Value},
};

use crate::{Executable, InterpreterContext, InterpreterError};

impl Executable for ExtendBitsInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        // println!("Executing ExtendBitsInstruction: {:?}", self);
        let stack_frame = ctx.stack.last_mut().unwrap();
        let in1_u64 = stack_frame.vars.get(self.in1).as_u64();
        let mask = (1_u64 << self.input_size) - 1;
        let sign_mask = 1_u64 << (self.input_size - 1);
        let in1_masked = in1_u64 & mask;
        let sign_bit = (in1_u64 & sign_mask) >> (self.input_size - 1);
        let ext_ones = !mask;
        let res_u64 = in1_masked | (ext_ones * sign_bit);
        let res = Value::Number(Number::trans_from_u64(res_u64, &self.in1_type));

        stack_frame.vars.set(self.out1, res.into());

        Ok(())
    }
}
