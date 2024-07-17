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

        let num1 = stack_frame.vars.get(self.lhs).to_number(&self.types);
        let num2 = stack_frame.vars.get(self.rhs).to_number(&self.types);

        // https://webassembly.github.io/spec/core/exec/numerics.html#nan-propagation
        if self.op != FBinaryOp::Copysign && (num1.is_nan() || num2.is_nan()) {
            let canonical_nan: u64 = Number::nan(&self.types).trans_to_u64();
            stack_frame.vars.set(self.out1, canonical_nan);
            return Ok(());
        }

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
