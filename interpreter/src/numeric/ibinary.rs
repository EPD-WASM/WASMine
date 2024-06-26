use wasm_types::IBinaryOp;
use {
    crate::{Executable, InterpreterContext, InterpreterError},
    ir::instructions::IBinaryInstruction,
    ir::structs::value::Number,
};

impl Executable for IBinaryInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        let stack_frame = ctx.stack.last_mut().unwrap();

        let in1_u64 = stack_frame.vars.get(self.lhs);
        let in2_u64 = stack_frame.vars.get(self.rhs);

        let types = todo!();
        let in1 = Number::trans_from_u64(in1_u64, &types);
        let in2 = Number::trans_from_u64(in2_u64, &types);

        let res: Number = match self.op {
            IBinaryOp::Add => in1 + in2,
            IBinaryOp::Sub => in1 - in2,
            IBinaryOp::Mul => in1 * in2,
            IBinaryOp::DivS => in1.as_signed() / in2.as_signed(),
            IBinaryOp::DivU => in1.as_unsigned() / in2.as_unsigned(),
            IBinaryOp::RemS => in1.as_signed() % in2.as_signed(),
            IBinaryOp::RemU => in2.as_unsigned() % in2.as_unsigned(),
            IBinaryOp::And => in1 & in2,
            IBinaryOp::Or => in1 | in2,
            IBinaryOp::Xor => in1 ^ in2,
            IBinaryOp::Shl => in1 << in2,
            IBinaryOp::ShrS => in1.as_signed() >> in2.as_signed(),
            IBinaryOp::ShrU => in1.as_unsigned() >> in2.as_unsigned(),
            IBinaryOp::Rotl => in1.rotate_left(in2),
            IBinaryOp::Rotr => in1.rotate_right(in2),
        };

        let res_u64 = res.trans_to_u64();
        stack_frame.vars.set(self.out1, res_u64);

        Ok(())
    }
}
