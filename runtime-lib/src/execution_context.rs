use crate::helpers::trap;
use crate::WASM_MAX_ADDRESS;
use crate::{error::RuntimeError, memory::MemoryStorage, runtime::Runtime};
use ir::structs::data::DataMode;
use ir::structs::module::Module as WasmModule;
use ir::structs::value::{Number, Value};
use std::rc::Rc;
use wasm_types::{DataIdx, MemIdx};

#[repr(transparent)]
pub(crate) struct ExecutionContextWrapper<'a>(
    pub(crate) &'a mut runtime_interface::ExecutionContext,
);

impl ExecutionContextWrapper<'_> {
    pub(crate) fn init(
        id: u32,
        runtime: *mut Runtime,
        module: &Rc<WasmModule>,
        imported_memories: &[runtime_interface::MemoryInstance],
    ) -> Result<runtime_interface::ExecutionContext, RuntimeError> {
        let memories = Box::new(MemoryStorage::new(module, imported_memories)?);
        for data in &module.datas {
            if let DataMode::Active { memory, offset } = &data.mode {
                if *memory != 0 {
                    return Err(RuntimeError::Msg(
                        "Preload data must be for memory 0".to_string(),
                    ));
                }
                let memory = &memories.0[*memory as usize];
                let offset = match offset {
                    Value::Number(Number::I32(offset)) => offset,
                    _ => return Err(RuntimeError::Msg(format!("Invalid offset: {:}", offset))),
                };
                memory.init(runtime, data.init.as_slice(), *offset, 0, None)?;
                // TODO: drop data
            }
        }

        let (memories_ptr, memories_len, memories_cap) = memories.into_raw_parts();
        Ok(runtime_interface::ExecutionContext {
            id,
            runtime: runtime as *mut std::ffi::c_void,
            recursion_size: 0,
            memories_ptr,
            memories_len,
            memories_cap,
        })
    }
}

impl ExecutionContextWrapper<'_> {
    pub(crate) fn get_memories_clone(&self) -> std::mem::ManuallyDrop<MemoryStorage> {
        MemoryStorage::from_raw_parts(
            self.0.memories_ptr,
            self.0.memories_len,
            self.0.memories_cap,
        )
    }
}

// implement functions from runtime-interface
#[no_mangle]
extern "C" fn memory_grow(
    ctxt: &runtime_interface::ExecutionContext,
    memory_idx: usize,
    grow_by: u32,
) -> i32 {
    let mut memories =
        MemoryStorage::from_raw_parts(ctxt.memories_ptr, ctxt.memories_len, ctxt.memories_cap);
    let rt_ptr = ctxt.runtime as *mut Runtime;
    let memory = &mut memories.0[memory_idx];
    let max_memory_size = unsafe { (*rt_ptr).module.memories[memory_idx].limits.max }
        .unwrap_or(WASM_MAX_ADDRESS as u32);
    memory.grow(grow_by, max_memory_size)
}

#[no_mangle]
extern "C" fn memory_fill(
    ctxt: &runtime_interface::ExecutionContext,
    memory_idx: usize,
    offset: usize,
    size: usize,
    value: u8,
) {
    let memories =
        MemoryStorage::from_raw_parts(ctxt.memories_ptr, ctxt.memories_len, ctxt.memories_cap);
    let memory = &memories.0[memory_idx];
    memory.fill(offset, size, value)
}

#[no_mangle]
extern "C" fn memory_copy(
    ctxt: &runtime_interface::ExecutionContext,
    memory_idx: MemIdx,
    src_offset: usize,
    dst_offset: usize,
    size: usize,
) {
    let memories =
        MemoryStorage::from_raw_parts(ctxt.memories_ptr, ctxt.memories_len, ctxt.memories_cap);
    let memory = &memories.0[memory_idx as usize];
    memory.copy(src_offset, dst_offset, size)
}

#[no_mangle]
extern "C" fn memory_init(
    ctxt: &runtime_interface::ExecutionContext,
    memory_idx: MemIdx,
    data_idx: DataIdx,
    src_offset: u32,
    dst_offset: u32,
    size: u32,
) {
    let memories =
        MemoryStorage::from_raw_parts(ctxt.memories_ptr, ctxt.memories_len, ctxt.memories_cap);
    let memory = &memories.0[memory_idx as usize];
    let rt_ref = ctxt.runtime as *mut Runtime;
    if let Err(e) = memory.rt_init(rt_ref, data_idx, src_offset, dst_offset, size) {
        log::error!("Error initializing memory: {:?}", e);
        trap();
    }
}
