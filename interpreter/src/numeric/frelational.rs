use ir::{instructions::FRelationalInstruction, structs::value::Number};
use wasm_types::FRelationalOp;

use crate::Executable;

impl Executable for FRelationalInstruction {
    fn execute(
        &mut self,
        ctx: &mut crate::InterpreterContext,
    ) -> Result<(), crate::InterpreterError> {
        let stack_frame = ctx.stack.last_mut().unwrap();

        let in1_u64 = stack_frame.vars.get(self.in1);
        let in2_u64 = stack_frame.vars.get(self.in2);

        let in1 = Number::trans_from_u64(in1_u64, &self.input_types);
        let in2 = Number::trans_from_u64(in2_u64, &self.input_types);

        let res = match self.op {
            FRelationalOp::Eq => in1 == in2,
            FRelationalOp::Ne => in1 != in2,
            FRelationalOp::Lt => in1 < in2,
            FRelationalOp::Gt => in1 > in2,
            FRelationalOp::Le => in1 <= in2,
            FRelationalOp::Ge => in1 >= in2,
        };

        let res_u64 = res as u64;

        stack_frame.vars.set(self.out1, res_u64);

        Ok(())
    }
}
