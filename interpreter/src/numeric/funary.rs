use wasm_types::FUnaryOp;
use {
    crate::{Executable, InterpreterContext, InterpreterError},
    ir::instructions::FUnaryInstruction,
    ir::utils::numeric_transmutes::Bit64,
};

impl Executable for FUnaryInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        let stack_frame = ctx.stack.last_mut().unwrap();

        let num1 = stack_frame.vars.get(self.in1).to_number(&self.types);
        let res1 = match self.op {
            FUnaryOp::Abs => num1.abs(),
            FUnaryOp::Neg => -num1,
            FUnaryOp::Sqrt => num1.sqrt(),
            FUnaryOp::Ceil => num1.ceil(),
            FUnaryOp::Floor => num1.floor(),
            FUnaryOp::Trunc => num1.trunc(),
            FUnaryOp::Nearest => num1.nearest(),
        };

        stack_frame.vars.set(self.out1, res1.trans_to_u64());

        Ok(())
    }
}
