use crate::memory::MemoryInstance;
use crate::WASM_MAX_ADDRESS;
use crate::{error::RuntimeError, memory::MemoryStorage};
use cee_scape::siglongjmp;
use core::slice;
use wasm_types::{DataIdx, MemIdx};

#[repr(transparent)]
pub(crate) struct ExecutionContextWrapper<'a>(
    pub(crate) &'a mut runtime_interface::ExecutionContext,
);

impl ExecutionContextWrapper<'_> {
    pub(crate) fn trap(&mut self, e: RuntimeError) {
        self.0.trap_msg = Some(e.to_string());
        unsafe {
            siglongjmp(self.0.trap_return.unwrap(), 1);
        }
    }
}

// implement functions from runtime-interface
#[no_mangle]
extern "C" fn memory_grow(
    ctxt: &runtime_interface::ExecutionContext,
    memory_idx: usize,
    grow_by: u32,
) -> i32 {
    let memories = MemoryStorage(unsafe {
        slice::from_raw_parts_mut(ctxt.memories_ptr as *mut MemoryInstance, ctxt.memories_len)
    });
    let memory = &mut memories.0[memory_idx];
    let max_memory_size = ctxt.wasm_module.memories[memory_idx]
        .limits
        .max
        .unwrap_or(WASM_MAX_ADDRESS as u32);
    memory.grow(grow_by, max_memory_size)
}

#[no_mangle]
extern "C" fn memory_fill(
    ctxt: &mut runtime_interface::ExecutionContext,
    memory_idx: usize,
    offset: usize,
    size: usize,
    value: u8,
) {
    let memories = MemoryStorage(unsafe {
        slice::from_raw_parts_mut(ctxt.memories_ptr as *mut MemoryInstance, ctxt.memories_len)
    });
    let memory = &memories.0[memory_idx];
    memory.fill(offset, size, value)
}

#[no_mangle]
extern "C" fn memory_copy(
    ctxt: &mut runtime_interface::ExecutionContext,
    memory_idx: MemIdx,
    src_offset: usize,
    dst_offset: usize,
    size: usize,
) {
    let memories = MemoryStorage(unsafe {
        slice::from_raw_parts_mut(ctxt.memories_ptr as *mut MemoryInstance, ctxt.memories_len)
    });
    let memory = &memories.0[memory_idx as usize];
    memory.copy(src_offset, dst_offset, size)
}

#[no_mangle]
extern "C" fn memory_init(
    ctxt: &mut runtime_interface::ExecutionContext,
    memory_idx: MemIdx,
    data_idx: DataIdx,
    src_offset: u32,
    dst_offset: u32,
    size: u32,
) {
    let memories = MemoryStorage(unsafe {
        slice::from_raw_parts_mut(ctxt.memories_ptr as *mut MemoryInstance, ctxt.memories_len)
    });
    let memory = &memories.0[memory_idx as usize];
    if let Err(e) = memory.rt_init(
        ctxt.wasm_module.clone(),
        data_idx,
        src_offset,
        dst_offset,
        size,
    ) {
        log::error!("Error initializing memory: {:?}", e);
        ExecutionContextWrapper(ctxt).trap(e);
    }
}
