use crate::{context::RTMemory, error::RuntimeError, memory::MemoryStorage, runtime::Runtime};

/// The only top level datastructure always available to the executing WASM code
#[repr(C)]
pub struct ExecutionContext {
    id: u32,

    runtime: *mut Runtime,

    /// number of current recursion levels, used to prevent stack overflowing
    recursion_size: u32,

    memories: MemoryStorage,
}

impl ExecutionContext {
    pub(crate) fn init(
        id: u32,
        runtime: *mut Runtime,
        memories_meta: Vec<RTMemory>,
    ) -> Result<Self, RuntimeError> {
        Ok(Self {
            id,
            runtime,
            recursion_size: 0,
            memories: MemoryStorage::new(&memories_meta)?,
        })
    }
}

impl runtime_interface::ExecutionContext for ExecutionContext {
    extern "C" fn memory_grow(&self, memory_idx: usize, grow_by: u32) -> i32 {
        let memory = &self.memories[memory_idx];
        let max_memory_size = unsafe { (*self.runtime).config.memories[memory_idx].max_size };
        memory.grow(grow_by, max_memory_size)
    }
    extern "C" fn memory_fill(&self, memory_idx: usize, offset: usize, size: usize, value: u8) {
        let memory = &self.memories[memory_idx];
        memory.fill(offset, size, value)
    }
    extern "C" fn memory_copy(
        &self,
        memory_idx: usize,
        src_offset: usize,
        dst_offset: usize,
        size: usize,
    ) {
        let memory = &self.memories[memory_idx];
        memory.copy(src_offset, dst_offset, size)
    }
}
