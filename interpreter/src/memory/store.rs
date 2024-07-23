use crate::{Executable, InterpreterContext, InterpreterError};
use ir::{instructions::StoreInstruction, structs::value::ValueRaw};
use wasm_types::{NumType, StoreOp};

enum StoreSize {
    Byte,
    Word,
    DoubleWord,
    Full,
}

impl Executable for StoreInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        // println!("Executing StoreInstruction {:?}", self);

        let stack_frame = ctx.stack.last_mut().unwrap();
        let dyn_addr = stack_frame.vars.get(self.addr_in).as_u32() as usize;
        // println!("offset: {:?}", self.memarg.offset);
        // println!("dyn_addr: {:?}", dyn_addr);
        let effective_address = dyn_addr as usize + self.memarg.offset as usize;
        // let effective_address = self.memarg.offset as usize;

        // println!("effective_address: {:?}", effective_address);
        let value = stack_frame.vars.get(self.value_in);

        // TODO access memory
        match self.operation {
            StoreOp::INNStore | StoreOp::FNNStore => {
                handle_store(ctx, effective_address, value, self.in_type, StoreSize::Full)
            }
            StoreOp::INNStore8 => {
                handle_store(ctx, effective_address, value, self.in_type, StoreSize::Byte)
            }
            StoreOp::INNStore16 => {
                handle_store(ctx, effective_address, value, self.in_type, StoreSize::Word)
            }
            StoreOp::INNStore32 => handle_store(
                ctx,
                effective_address,
                value,
                self.in_type,
                StoreSize::DoubleWord,
            ),
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
) {
    let memory_inst_ptr = ctx.exec_ctx.memories_ptr;
    let memory_data_ptr = unsafe { (*memory_inst_ptr).data };
    let memory_data_slice = unsafe {
        core::slice::from_raw_parts_mut(memory_data_ptr, (*memory_inst_ptr).size as usize * 65536)
    };
    // println!("memory_data_ptr: {:?}", memory_data_ptr);
    // println!("addr: {:?}", addr);
    // println!("memory size: {:?} pages", unsafe {
    //     (*memory_inst_ptr).size
    // });
    // println!("memory instance: {:?}", unsafe { &*memory_inst_ptr });

    debug_assert!(
        addr < unsafe { (*memory_inst_ptr).size } as usize * 65536,
        "Memory access out of bounds: {} >= {}",
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

    let val_slice = &value.as_v128()[..num_bytes_to_store];
    let dst_slice = &mut memory_data_slice[addr..addr + num_bytes_to_store];

    // println!("Storing {} bytes to memory", num_bytes_to_store);
    dst_slice.copy_from_slice(val_slice);
}
