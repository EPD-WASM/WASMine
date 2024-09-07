use ir::instructions::IRelationalInstruction;
use wasm_types::IRelationalOp;

use crate::{Executable, InterpreterError};

impl Executable for IRelationalInstruction {
    fn execute(&mut self, ctx: &mut crate::InterpreterContext) -> Result<(), InterpreterError> {
        let stack_frame = ctx.stack.last_mut().unwrap();

        let in1 = stack_frame.vars.get_number(self.in1, self.input_types);
        let in2 = stack_frame.vars.get_number(self.in2, self.input_types);

        let res = match self.op {
            IRelationalOp::Eq => in1 == in2,
            IRelationalOp::Ne => in1 != in2,
            IRelationalOp::LtS => in1.as_signed() < in2.as_signed(),
            IRelationalOp::LtU => in1.as_unsigned() < in2.as_unsigned(),
            IRelationalOp::GtS => in1.as_signed() > in2.as_signed(),
            IRelationalOp::GtU => in1.as_unsigned() > in2.as_unsigned(),
            IRelationalOp::LeS => in1.as_signed() <= in2.as_signed(),
            IRelationalOp::LeU => in1.as_unsigned() <= in2.as_unsigned(),
            IRelationalOp::GeS => in1.as_signed() >= in2.as_signed(),
            IRelationalOp::GeU => in1.as_unsigned() >= in2.as_unsigned(),
        };

        let res_u64 = res as u64;

        stack_frame.vars.set(self.out1, res_u64.into());

        Ok(())
    }
}
