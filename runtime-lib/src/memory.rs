use crate::error::RuntimeError;
use crate::linker::RTImport;
use crate::module_instance::InstantiationError;
use crate::{Cluster, WASM_PAGE_SIZE, WASM_RESERVED_MEMORY_SIZE};
use ir::structs::data::{Data, DataMode};
use ir::structs::memory::Memory;
use ir::structs::module::Module as WasmModule;
use ir::structs::value::{Number, Value};
use nix::errno::Errno;
use nix::libc::mprotect;
use nix::{errno, libc};
use std::ops::Index;
use std::rc::Rc;
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
        data_source: &[u8],
        src_offset: u32,
        dst_offset: u32,
        size: Option<u32>,
    ) -> Result<(), InstantiationError> {
        unsafe {
            let size = size
                .map(|s| s as u64)
                .unwrap_or((data_source.len().saturating_sub(src_offset as usize)) as u64);
            if src_offset as u64 + size > data_source.len() as u64 {
                return Err(InstantiationError::DataSourceOOB);
            }
            if dst_offset as u64 + size > self.0.size as u64 * WASM_PAGE_SIZE as u64 {
                return Err(InstantiationError::MemoryInitOOB);
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
        wasm_module: Rc<WasmModule>,
        data_idx: DataIdx,
        src_offset: u32,
        dst_offset: u32,
        size: u32,
    ) -> Result<(), RuntimeError> {
        let data_instance = match wasm_module.datas.get(data_idx as usize) {
            None => {
                return Err(RuntimeError::Msg(format!(
                    "Data instance not found: {}",
                    data_idx
                )));
            }
            Some(data_instance) => data_instance,
        };
        if data_instance.mode != DataMode::Passive {
            return Err(RuntimeError::Msg(format!(
                "Cannot initialize memory using active data segment: {}",
                data_idx
            )));
        }
        if let Err(e) = self.init(&data_instance.init, src_offset, dst_offset, Some(size)) {
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

pub(crate) struct MemoryStorage<'a>(pub(crate) &'a mut [MemoryInstance]);

impl<'a> MemoryStorage<'a> {
    pub(crate) fn init_on_cluster(
        cluster: &'a Cluster,
        memories_meta: &[Memory],
        data_meta: &[Data],
        imports: &[RTImport],
    ) -> Result<&'a mut [MemoryInstance], InstantiationError> {
        let mut memories = Vec::with_capacity(1);
        for memory_desc in memories_meta.iter() {
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
                return Err(InstantiationError::AllocationFailure(Errno::last()));
            }

            if 0 != unsafe {
                libc::mprotect(
                    memory_ptr,
                    (memory_desc.limits.min * WASM_PAGE_SIZE) as usize,
                    libc::PROT_READ | libc::PROT_WRITE,
                )
            } {
                return Err(InstantiationError::AllocationFailure(Errno::last()));
            }
            memories.push(MemoryInstance::new(
                memory_ptr as *mut u8,
                memory_desc.limits.min,
            ))
        }

        for data in data_meta {
            if let DataMode::Active { memory, offset } = &data.mode {
                if *memory != 0 {
                    return Err(InstantiationError::MemoryIdxNotZero);
                }
                let memory = &memories[*memory as usize];
                let offset = match offset {
                    Value::Number(Number::I32(offset)) => offset,
                    _ => return Err(InstantiationError::InvalidDataOffsetType(offset.r#type())),
                };
                memory.init(data.init.as_slice(), *offset, 0, None)?;
                // TODO: drop data
            }
        }

        Ok(cluster.alloc_memories(memories))
    }
}

impl Index<usize> for MemoryStorage<'_> {
    type Output = MemoryInstance;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}
