use ir::{instructions::ITestInstruction, structs::value::Number};

use crate::Executable;

impl Executable for ITestInstruction {
    fn execute(
        &mut self,
        ctx: &mut crate::InterpreterContext,
    ) -> Result<(), crate::InterpreterError> {
        let stack_frame = ctx.stack.last_mut().unwrap();

        let in1_u64 = stack_frame.vars.get(self.in1);

        let in1 = Number::trans_from_u64(in1_u64, &self.input_type);
        let zero = Number::trans_from_u64(0, &self.input_type);

        let res = match self.op {
            wasm_types::ITestOp::Eqz => in1 == zero,
        };

        let res_u64 = res as u64;

        stack_frame.vars.set(self.out1, res_u64);

        Ok(())
    }
}
