use ir::instructions::ExtendTypeInstruction;

use crate::{Executable, InterpreterContext, InterpreterError};

// "ExtendType" means extend i32 to i64
impl Executable for ExtendTypeInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        let stack_frame = ctx.stack.last_mut().unwrap();
        let in1_u64 = stack_frame.vars.get(self.in1);
        let ext_ones: u64 = 0xFFFF_FFFF_0000_0000;
        let is_negative = in1_u64 & 0x8000_0000 != 0;
        let res = in1_u64 | (ext_ones * (self.signed && is_negative) as u64);

        stack_frame.vars.set(self.out1, res);

        Ok(())
    }
}
