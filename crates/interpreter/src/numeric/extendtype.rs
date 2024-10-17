use module::instructions::ExtendTypeInstruction;

use crate::{Executable, InterpreterContext, InterpreterError};

// "ExtendType" means extend i32 to i64
impl Executable for ExtendTypeInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        let stack_frame = ctx.stack.last_mut().unwrap();
        let in1_u64 = stack_frame.vars.get(self.in1).as_u64();
        let res = if self.signed {
            in1_u64 as i32 as i64
        } else {
            in1_u64 as u32 as i64
        };
        stack_frame.vars.set(self.out1, res.into());
        Ok(())
    }
}
