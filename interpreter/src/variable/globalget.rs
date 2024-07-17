use ir::{instructions::GlobalGetInstruction, utils::numeric_transmutes::Bit64};

use crate::{Executable, InterpreterContext, InterpreterError};

impl Executable for GlobalGetInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        // println!("Executing GlobalGetInstruction {:?}", self);

        let stack_frame = ctx.stack.last_mut().unwrap();
        let global_storage = unsafe { &*ctx.exec_ctx.globals_ptr };

        debug_assert!(
            self.global_idx < global_storage.globals.len() as u32,
            "Global index out of bounds: {} >= {}",
            self.global_idx,
            global_storage.globals.len()
        );

        let global_instance = &global_storage.globals[self.global_idx as usize];
        let global_val = unsafe { *global_instance.addr };

        stack_frame.vars.set(self.out1, global_val);

        Ok(())
    }
}
