use std::ops::Index;

use crate::context::RTMemory;
use crate::error::RuntimeError;
use crate::{WASM_PAGE_SIZE, WASM_RESERVED_MEMORY_SIZE};
use nix::errno::Errno;
use nix::libc::mprotect;
use nix::{errno, libc};

pub(crate) struct MemoryInstance {
    data: *mut u8,
    size: u32,
}

impl MemoryInstance {
    pub(crate) fn new(data: *mut u8, size: u32) -> Self {
        Self { data, size }
    }

    pub(crate) fn grow(&self, grow_by: u32, max_memory_size: u32) -> i32 {
        // larger than 2**32 = 4GiB?
        if self.size + grow_by > 2_u32.pow(16) {
            return -1;
        }
        // larger than memory::limits::max_size?
        if self.size + grow_by > max_memory_size {
            return -1;
        }

        // increase size!
        let new_size = (self.size + grow_by) * WASM_PAGE_SIZE;
        let res = unsafe {
            mprotect(
                self.data as *mut libc::c_void,
                new_size as usize,
                libc::PROT_READ | libc::PROT_WRITE,
            )
        };
        if res != 0 {
            println!("Memory grow failed: {}", errno::Errno::last());
            return -1;
        }
        self.size as i32
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
        if unsafe { libc::munmap(self.data as *mut libc::c_void, WASM_RESERVED_MEMORY_SIZE) } != 0 {
            println!(
                "Failed to unmap memory at 0x{:x}: {}",
                self.data as usize,
                Errno::last()
            );
        }
    }
}

pub(crate) struct MemoryStorage(Vec<MemoryInstance>);

impl MemoryStorage {
    pub(crate) fn new(memories_meta: &Vec<RTMemory>) -> Result<Self, RuntimeError> {
        let mut memories = Vec::new();
        for memory_desc in memories_meta {
            let memory_ptr = unsafe {
                libc::mmap(
                    core::ptr::null_mut::<libc::c_void>(),
                    WASM_RESERVED_MEMORY_SIZE,
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
}

impl Index<usize> for MemoryStorage {
    type Output = MemoryInstance;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}
