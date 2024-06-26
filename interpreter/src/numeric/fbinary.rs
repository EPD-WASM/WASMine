use {
    crate::{Executable, InterpreterContext, InterpreterError},
    ir::instructions::FBinaryInstruction,
    ir::structs::value::Number,
    ir::utils::numeric_transmutes::Bit64,
};

use wasm_types::FBinaryOp;
impl Executable for FBinaryInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        let stack_frame = ctx.stack.last_mut().unwrap();

        let types = todo!();

        let num1 = stack_frame.vars.get(self.lhs).to_number(&types);
        let num2 = stack_frame.vars.get(self.rhs).to_number(&types);

        let res1 = match self.op {
            FBinaryOp::Add => num1 + num2,
            FBinaryOp::Sub => num1 - num2,
            FBinaryOp::Mul => num1 * num2,
            FBinaryOp::Div => num1 / num2,
            FBinaryOp::Min => Number::min(&num1, &num2),
            FBinaryOp::Max => Number::max(&num1, &num2),
            FBinaryOp::Copysign => num1.copysign(&num2),
        };

        stack_frame.vars.set(self.out1, res1.trans_to_u64());

        Ok(())
    }
}
