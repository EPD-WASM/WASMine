use crate::{Executable, InterpreterContext, InterpreterError};
use module::{
    instructions::TruncInstruction,
    objects::value::{Number, ValueRaw},
    utils::numeric_transmutes::{Bit128, Bit64},
};
use wasm_types::NumType;

impl Executable for TruncInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        let stack_frame = ctx.stack.last_mut().unwrap();
        let in1 = stack_frame.vars.get_number(self.in1, self.in1_type);
        let in1_float = match in1 {
            Number::F32(n) => n as f64,
            Number::F64(n) => n,
            a => panic!("Invalid in type for trunc: {}", a),
        };

        let in1_trunc = in1_float.trunc();

        log::trace!("Truncated {} to {}", in1_float, in1_trunc);

        let max = match (self.out1_type, self.signed) {
            (NumType::I32, false) => u32::MAX as f64,
            (NumType::I64, false) => u64::MAX as f64,
            (NumType::I32, true) => i32::MAX as f64,
            (NumType::I64, true) => i64::MAX as f64,
            (a, _) => panic!("Invalid out type for trunc: {}", a),
        };

        let min = match (self.out1_type, self.signed) {
            (NumType::I32, false) => 0.0,
            (NumType::I64, false) => 0.0,
            (NumType::I32, true) => i32::MIN as f64,
            (NumType::I64, true) => i64::MIN as f64,
            (a, _) => panic!("Invalid out type for trunc: {}", a),
        };

        if in1_trunc.is_nan() || in1_trunc.is_infinite() || in1_trunc < min || in1_trunc > max {
            // undefined
            return Err(InterpreterError::TruncError);
        }

        let res_u64 = match self.out1_type {
            NumType::I32 => (in1_trunc as i64).trans_u64(),
            NumType::I64 => (in1_trunc as i128).trans_u64(),
            a => panic!("Invalid out type for trunc: {}", a),
        };

        stack_frame.vars.set(self.out1, res_u64.into());

        Ok(())
    }
}
