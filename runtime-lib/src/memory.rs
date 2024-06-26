use std::mem::ManuallyDrop;
use std::ops::Index;

use crate::context::RTMemory;
use crate::error::RuntimeError;
use crate::{WASM_PAGE_SIZE, WASM_RESERVED_MEMORY_SIZE};
use nix::errno::Errno;
use nix::libc::mprotect;
use nix::{errno, libc};

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
    pub(crate) fn new(memories_meta: &Vec<RTMemory>) -> Result<Self, RuntimeError> {
        let mut memories = Vec::with_capacity(1);
        for memory_desc in memories_meta {
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
                    (memory_desc.min_size * WASM_PAGE_SIZE) as usize,
                    libc::PROT_READ | libc::PROT_WRITE,
                )
            } {
                return Err(RuntimeError::AllocationFailure(Errno::last()));
            }
            memories.push(MemoryInstance::new(
                memory_ptr as *mut u8,
                memory_desc.min_size,
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
