use ir::instructions::GlobalSetInstruction;
use wasm_types::{GlobalType, ValType};

use crate::{Executable, InterpreterContext, InterpreterError};

impl Executable for GlobalSetInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        let stack_frame = ctx.stack.last_mut().unwrap();

        let in1_u64 = stack_frame.vars.get(self.in1);

        let global_storage = unsafe { &*ctx.exec_ctx.globals_ptr };
        let global_instance = &global_storage.globals[self.global_idx as usize];
        unsafe { *global_instance.addr.as_ptr() = in1_u64 };

        Ok(())
    }
}
