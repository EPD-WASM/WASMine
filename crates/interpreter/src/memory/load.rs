use ir::{
    instructions::LoadInstruction, structs::value::ValueRaw, utils::numeric_transmutes::Bit64,
};
use wasm_types::{LoadOp, NumType};

use crate::{Executable, InterpreterContext, InterpreterError};

enum LoadSize {
    Byte,
    Word,
    DoubleWord,
    Full,
}

impl Executable for LoadInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        // println!("Executing LoadInstruction {:?}", self);

        let stack_frame = ctx.stack.last_mut().unwrap();
        let dyn_addr = stack_frame.vars.get(self.addr).as_u32() as usize;
        let effective_address = dyn_addr as usize + self.memarg.offset as usize;
        // let effective_address = self.memarg.offset as usize;

        // println!("effective_address: {:?}", effective_address);
        // TODO access memory
        let res = match self.operation {
            LoadOp::INNLoad | LoadOp::FNNLoad => {
                handle_load(ctx, effective_address, self.out1_type, LoadSize::Full, None)
            }
            LoadOp::INNLoad8U => handle_load(
                ctx,
                effective_address,
                self.out1_type,
                LoadSize::Byte,
                Some(false),
            ),
            LoadOp::INNLoad8S => handle_load(
                ctx,
                effective_address,
                self.out1_type,
                LoadSize::Byte,
                Some(true),
            ),
            LoadOp::INNLoad16U => handle_load(
                ctx,
                effective_address,
                self.out1_type,
                LoadSize::Word,
                Some(false),
            ),
            LoadOp::INNLoad16S => handle_load(
                ctx,
                effective_address,
                self.out1_type,
                LoadSize::Word,
                Some(true),
            ),
            LoadOp::INNLoad32U => handle_load(
                ctx,
                effective_address,
                self.out1_type,
                LoadSize::DoubleWord,
                Some(false),
            ),
            LoadOp::INNLoad32S => handle_load(
                ctx,
                effective_address,
                self.out1_type,
                LoadSize::DoubleWord,
                Some(true),
            ),
        };

        let stack_frame = ctx.stack.last_mut().unwrap();
        stack_frame.vars.set(self.out1, res);

        Ok(())
    }
}

fn handle_load(
    ctx: &mut InterpreterContext,
    addr: usize,
    out_type: NumType,
    size: LoadSize,
    signed: Option<bool>,
) -> ValueRaw {
    let memory_inst_ptr = ctx.exec_ctx.memories_ptr;
    let memory_data_ptr = unsafe { (*memory_inst_ptr).data };
    let src_val_ptr = unsafe { memory_data_ptr.add(addr) };

    let num_bytes_to_load = match (&size, out_type) {
        (LoadSize::Byte, _) => 1,
        (LoadSize::Word, _) => 2,
        (LoadSize::DoubleWord, _) => 4,
        (LoadSize::Full, NumType::I32) => 4,
        (LoadSize::Full, NumType::I64) => 8,
        (LoadSize::Full, NumType::F32) => 4,
        (LoadSize::Full, NumType::F64) => 8,
    };

    // println!("Loading {} bytes from memory", num_bytes_to_load,);
    let bytes;
    unsafe {
        bytes = core::slice::from_raw_parts(src_val_ptr, num_bytes_to_load);
    }
    let mut padded: [u8; 8] = [0; 8];
    padded[..num_bytes_to_load].copy_from_slice(bytes);

    let res = if signed == Some(true) {
        let res_signed = match num_bytes_to_load {
            1 => i8::from_le_bytes(bytes.try_into().unwrap()) as i64,
            2 => i16::from_le_bytes(bytes.try_into().unwrap()) as i64,
            4 => i32::from_le_bytes(bytes.try_into().unwrap()) as i64,
            8 => i64::from_le_bytes(padded),
            _ => unreachable!(),
        };
        res_signed.trans_u64()
    } else {
        u64::from_le_bytes(padded)
    };

    // println!("Loaded value: {}", res);
    ValueRaw::u64(res)
}
