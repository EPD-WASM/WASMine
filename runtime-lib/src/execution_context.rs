use crate::{context::RTMemory, error::RuntimeError, memory::MemoryStorage, runtime::Runtime};

#[repr(transparent)]
pub(crate) struct ExecutionContext(pub(crate) runtime_interface::ExecutionContext);

impl ExecutionContext {
    pub(crate) fn init(
        id: u32,
        runtime: *mut Runtime,
        memories_meta: Vec<RTMemory>,
    ) -> Result<Self, RuntimeError> {
        let memories = Box::new(MemoryStorage::new(&memories_meta)?);
        let (memories_ptr, memories_len, memories_cap) = memories.into_raw_parts();
        Ok(Self(runtime_interface::ExecutionContext {
            id,
            runtime: runtime as *mut std::ffi::c_void,
            recursion_size: 0,
            memories_ptr,
            memories_len,
            memories_cap,
        }))
    }
}

extern "C" fn memory_grow(ctxt: &ExecutionContext, memory_idx: usize, grow_by: u32) -> i32 {
    let memories = MemoryStorage::from_raw_parts(
        ctxt.0.memories_ptr,
        ctxt.0.memories_len,
        ctxt.0.memories_cap,
    );
    let rt_ptr = ctxt.0.runtime as *mut Runtime;
    let memory = &memories.0[memory_idx];
    let max_memory_size = unsafe { (*rt_ptr).config.memories[memory_idx].max_size };
    memory.grow(grow_by, max_memory_size)
}

extern "C" fn memory_fill(
    ctxt: &ExecutionContext,
    memory_idx: usize,
    offset: usize,
    size: usize,
    value: u8,
) {
    let memories = MemoryStorage::from_raw_parts(
        ctxt.0.memories_ptr,
        ctxt.0.memories_len,
        ctxt.0.memories_cap,
    );
    let memory = &memories.0[memory_idx];
    memory.fill(offset, size, value)
}

extern "C" fn memory_copy(
    ctxt: &ExecutionContext,
    memory_idx: usize,
    src_offset: usize,
    dst_offset: usize,
    size: usize,
) {
    let memories = MemoryStorage::from_raw_parts(
        ctxt.0.memories_ptr,
        ctxt.0.memories_len,
        ctxt.0.memories_cap,
    );
    let memory = &memories.0[memory_idx];
    memory.copy(src_offset, dst_offset, size)
}
