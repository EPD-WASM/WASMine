use crate::{Executable, InterpreterContext, InterpreterError};
use log::trace;
use module::{
    instructions::StoreInstruction,
    objects::value::{Number, Value, ValueRaw},
};
use wasm_types::{NumType, StoreOp, ValType};

enum StoreSize {
    Byte,
    Word,
    DoubleWord,
    Full,
}

impl Executable for StoreInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        let stack_frame = ctx.stack.last_mut().unwrap();
        let dyn_addr_raw = stack_frame.vars.get(self.addr_in);
        let dyn_addr = dyn_addr_raw.as_i32();
        let effective_address = (dyn_addr as i64 + self.memarg.offset as i64) as usize;

        log::trace!("Memory Store: {:#?}", self);
        log::trace!("Dynamic addr: {}", dyn_addr);
        log::trace!("Dynamic addr.as_i32: {}", dyn_addr);
        log::trace!("Offset: {}", self.memarg.offset);
        log::trace!("Effective address: {}", effective_address);

        let value = stack_frame.vars.get(self.value_in);

        match self.operation {
            StoreOp::INNStore | StoreOp::FNNStore => {
                handle_store(ctx, effective_address, value, self.in_type, StoreSize::Full)?
            }
            StoreOp::INNStore8 => {
                handle_store(ctx, effective_address, value, self.in_type, StoreSize::Byte)?
            }
            StoreOp::INNStore16 => {
                handle_store(ctx, effective_address, value, self.in_type, StoreSize::Word)?
            }
            StoreOp::INNStore32 => handle_store(
                ctx,
                effective_address,
                value,
                self.in_type,
                StoreSize::DoubleWord,
            )?,
        }

        Ok(())
    }
}

fn handle_store(
    ctx: &mut InterpreterContext,
    addr: usize,
    value: ValueRaw,
    in_type: NumType,
    size: StoreSize,
) -> Result<(), InterpreterError> {
    let memory_inst_ptr = ctx.exec_ctx.memories_ptr;
    let memory_data_ptr = unsafe { (*memory_inst_ptr).data };
    let memory_data_slice = unsafe {
        core::slice::from_raw_parts_mut(memory_data_ptr, (*memory_inst_ptr).size as usize * 65536)
    };

    debug_assert!(
        addr <= unsafe { (*memory_inst_ptr).size } as usize * 65536,
        "Memory access out of bounds: {} > {}",
        addr,
        unsafe { (*memory_inst_ptr).size } as usize * 65536
    );

    let num_bytes_to_store = match (size, in_type) {
        (StoreSize::Byte, _) => 1,
        (StoreSize::Word, _) => 2,
        (StoreSize::DoubleWord, _) => 4,
        (StoreSize::Full, NumType::I32) => 4,
        (StoreSize::Full, NumType::I64) => 8,
        (StoreSize::Full, NumType::F32) => 4,
        (StoreSize::Full, NumType::F64) => 8,
    };

    trace!("Storing {} bytes at address {}", num_bytes_to_store, addr);
    trace!(
        "Value: {:?}",
        Value::from_raw(value, ValType::Number(in_type))
    );

    if addr + num_bytes_to_store > memory_data_slice.len() {
        return Err(InterpreterError::IdxBounds);
    }

    let val_slice = &value.as_v128()[..num_bytes_to_store];

    let dst_slice = &mut memory_data_slice[addr..addr + num_bytes_to_store];

    dst_slice.copy_from_slice(val_slice);

    Ok(())
}
