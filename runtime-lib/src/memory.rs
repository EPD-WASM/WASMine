use crate::error::RuntimeError;
use crate::helpers::trap;
use crate::runtime::Runtime;
use crate::{WASM_PAGE_SIZE, WASM_RESERVED_MEMORY_SIZE};
use ir::structs::data::DataMode;
use ir::structs::module::Module;
use nix::errno::Errno;
use nix::libc::mprotect;
use nix::{errno, libc};
use std::mem::ManuallyDrop;
use std::ops::Index;
use wasm_types::DataIdx;

#[repr(transparent)]
pub(crate) struct MemoryInstance(pub(crate) runtime_interface::MemoryInstance);

impl MemoryInstance {
    pub(crate) fn new(data: *mut u8, size: u32) -> Self {
        Self(runtime_interface::MemoryInstance { data, size })
    }

    pub(crate) fn grow(&mut self, grow_by: u32, max_memory_size: u32) -> i32 {
        // larger than 2**32 = 4GiB?
        if self.0.size + grow_by > 2_u32.pow(16) {
            return -1;
        }
        // larger than memory::limits::max_size?
        if self.0.size + grow_by > max_memory_size {
            return -1;
        }

        // increase size!
        let new_size = (self.0.size + grow_by) * WASM_PAGE_SIZE;
        let res = unsafe {
            mprotect(
                self.0.data as *mut libc::c_void,
                new_size as usize,
                libc::PROT_READ | libc::PROT_WRITE,
            )
        };
        if res != 0 {
            println!("Memory grow failed: {}", errno::Errno::last());
            return -1;
        }
        let old_size = self.0.size;
        self.0.size += grow_by;
        old_size as i32
    }

    pub(crate) fn fill(&self, offset: usize, size: usize, value: u8) {
        unimplemented!()
    }

    pub(crate) fn copy(&self, src_offset: usize, dst_offset: usize, size: usize) {
        unimplemented!()
    }

    pub(crate) fn init(
        &self,
        rt: *mut Runtime,
        data_source: &[u8],
        src_offset: u32,
        dst_offset: u32,
        size: Option<u32>,
    ) -> Result<(), RuntimeError> {
        unsafe {
            let size = size
                .map(|s| s as u64)
                .unwrap_or((data_source.len().saturating_sub(src_offset as usize)) as u64);
            if src_offset as u64 + size > data_source.len() as u64 {
                return Err(RuntimeError::Msg("Data source too small.".into()));
            }
            if dst_offset as u64 + size > self.0.size as u64 * WASM_PAGE_SIZE as u64 {
                return Err(RuntimeError::Msg("Memory too small.".into()));
            }
            if size == 0 {
                return Ok(());
            }
            libc::memcpy(
                self.0.data.byte_add(dst_offset as usize) as *mut libc::c_void,
                data_source[src_offset as usize..].as_ptr() as *const libc::c_void,
                size as usize,
            );
            Ok(())
        }
    }

    pub(crate) fn rt_init(
        &self,
        rt: *mut Runtime,
        data_idx: DataIdx,
        src_offset: u32,
        dst_offset: u32,
        size: u32,
    ) -> Result<(), RuntimeError> {
        let data_instance = match unsafe { (*rt).module.datas.get(data_idx as usize) } {
            None => {
                log::error!("Data instance not found: {}", data_idx);
                trap();
            }
            Some(data_instance) => data_instance,
        };
        if data_instance.mode != DataMode::Passive {
            return Err(RuntimeError::Msg(format!(
                "Cannot initialize memory using active data segment: {}",
                data_idx
            )));
        }
        if let Err(e) = self.init(rt, &data_instance.init, src_offset, dst_offset, Some(size)) {
            return Err(RuntimeError::Msg(format!(
                "Failed to initialize memory: {}",
                e
            )));
        }
        Ok(())
    }
}

impl Drop for MemoryInstance {
    fn drop(&mut self) {
        if unsafe {
            libc::munmap(
                self.0.data as *mut libc::c_void,
                WASM_RESERVED_MEMORY_SIZE as usize,
            )
        } != 0
        {
            println!(
                "Failed to unmap memory at 0x{:x}: {}",
                self.0.data as usize,
                Errno::last()
            );
        }
    }
}

pub(crate) struct MemoryStorage(pub(crate) Vec<MemoryInstance>);

impl MemoryStorage {
    pub(crate) fn new(
        wasm_module: &Module,
        imported_memories: &[runtime_interface::MemoryInstance],
    ) -> Result<Self, RuntimeError> {
        let mut memories = Vec::with_capacity(1);
        for memory_desc in wasm_module.memories.iter() {
            let memory_ptr = unsafe {
                libc::mmap(
                    core::ptr::null_mut::<libc::c_void>(),
                    WASM_RESERVED_MEMORY_SIZE as usize,
                    libc::PROT_NONE,
                    libc::MAP_ANONYMOUS | libc::MAP_PRIVATE | libc::MAP_NORESERVE,
                    -1,
                    0,
                )
            };
            if memory_ptr == libc::MAP_FAILED {
                return Err(RuntimeError::AllocationFailure(Errno::last()));
            }

            if 0 != unsafe {
                libc::mprotect(
                    memory_ptr,
                    (memory_desc.limits.min * WASM_PAGE_SIZE) as usize,
                    libc::PROT_READ | libc::PROT_WRITE,
                )
            } {
                return Err(RuntimeError::AllocationFailure(Errno::last()));
            }
            memories.push(MemoryInstance::new(
                memory_ptr as *mut u8,
                memory_desc.limits.min,
            ))
        }
        Ok(Self(memories))
    }

    pub(crate) fn into_raw_parts(
        mut self,
    ) -> (*mut runtime_interface::MemoryInstance, usize, usize) {
        let length = self.0.len();
        let capacity = self.0.capacity();
        let ptr = self.0.as_mut_ptr() as *mut runtime_interface::MemoryInstance;
        std::mem::forget(self.0);
        (ptr, length, capacity)
    }

    pub(crate) fn from_raw_parts(
        ptr: *mut runtime_interface::MemoryInstance,
        len: usize,
        capacity: usize,
    ) -> ManuallyDrop<Self> {
        ManuallyDrop::new(Self(unsafe {
            Vec::from_raw_parts(ptr as *mut MemoryInstance, len, capacity)
        }))
    }
}

impl Index<usize> for MemoryStorage {
    type Output = MemoryInstance;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}
