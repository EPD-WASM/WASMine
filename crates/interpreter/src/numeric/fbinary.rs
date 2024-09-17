use {
    crate::{Executable, InterpreterContext, InterpreterError},
    module::{
        instructions::FBinaryInstruction,
        objects::value::{Number, Value},
    },
};

use wasm_types::FBinaryOp;
impl Executable for FBinaryInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        let stack_frame = ctx.stack.last_mut().unwrap();

        let num1 = stack_frame.vars.get_number(self.lhs, self.types);
        let num2 = stack_frame.vars.get_number(self.rhs, self.types);

        // https://webassembly.github.io/spec/core/exec/numerics.html#nan-propagation
        if self.op != FBinaryOp::Copysign && (num1.is_nan() || num2.is_nan()) {
            let canonical_nan: u64 = Number::nan(&self.types).trans_to_u64();
            stack_frame.vars.set(self.out1, canonical_nan.into());
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

        stack_frame.vars.set(self.out1, Value::Number(res1).into());

        Ok(())
    }
}
