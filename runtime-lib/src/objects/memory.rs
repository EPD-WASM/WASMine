use crate::{
    error::RuntimeError, linker::RTMemoryImport, objects::execution_context::trap_on_err, Cluster,
    WASM_PAGE_LIMIT, WASM_PAGE_SIZE, WASM_RESERVED_MEMORY_SIZE,
};
use core::slice;
use ir::structs::data::{Data, DataMode};
use ir::structs::memory::Memory;
use ir::structs::module::Module as WasmModule;
use ir::structs::value::{ConstantValue, Number, Value};
use nix::errno::Errno;
use nix::libc::mprotect;
use nix::{errno, libc};
use runtime_interface::GlobalStorage;
use std::ops::Index;
use std::rc::Rc;
use wasm_types::{DataIdx, MemIdx, ValType};

#[derive(Debug, thiserror::Error)]
pub enum MemoryError {
    #[error("Encountered reference to memory with index != 0. A maximum of one memory is allowed per wasm module.")]
    MemoryIdxNotZero,
    #[error("Offset into segment was of invalid type '{0:}'")]
    InvalidOffsetType(ValType),
    #[error("Supplied datasource too small.")]
    DataSourceOOB,
    #[error("Memory init data too large for memory.")]
    MemoryInitOOB,
    #[error("Memory fill range too large for memory.")]
    MemoryFillOOB,
    #[error("Memory copy range too large for memory.")]
    MemoryCopyOOB,
    #[error("Data segment index out of bounds.")]
    DataIdxOOB,
    #[error("Allocation Failure ({0})")]
    AllocationFailure(Errno),
}

#[repr(transparent)]
pub(crate) struct MemoryObject(pub(crate) runtime_interface::MemoryInstance);

impl MemoryObject {
    pub(crate) fn new(data: *mut u8, size: u32, max_size: u32) -> Self {
        Self(runtime_interface::MemoryInstance {
            data,
            size,
            max_size,
        })
    }

    pub(crate) fn grow(&mut self, grow_by: u32) -> i32 {
        if grow_by == 0 {
            return self.0.size as i32;
        }

        // larger than 2**32 = 4GiB?
        if self.0.size + grow_by > 2_u32.pow(16) {
            log::debug!("Memory grow failed: larger than 4GiB");
            return -1;
        }
        // larger than memory::limits::max_size?
        if self.0.size + grow_by > self.0.max_size {
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
            log::debug!("Memory grow failed: {}", errno::Errno::last());
            return -1;
        }
        let old_size = self.0.size;
        self.0.size += grow_by;
        old_size as i32
    }

    pub(crate) fn fill(&self, offset: u32, size: u32, value: u8) -> Result<(), MemoryError> {
        if offset + size > self.0.size * WASM_PAGE_SIZE {
            return Err(MemoryError::MemoryFillOOB);
        }
        if size == 0 {
            return Ok(());
        }
        unsafe {
            self.0
                .data
                .add(offset as usize)
                .write_bytes(value, size as usize)
        }
        Ok(())
    }

    pub(crate) fn copy(
        &self,
        src_offset: u32,
        dst_offset: u32,
        size: u32,
    ) -> Result<(), MemoryError> {
        if src_offset + size > self.0.size * WASM_PAGE_SIZE {
            return Err(MemoryError::MemoryCopyOOB);
        }
        if dst_offset + size > self.0.size * WASM_PAGE_SIZE {
            return Err(MemoryError::MemoryCopyOOB);
        }
        if size == 0 {
            return Ok(());
        }
        unsafe {
            std::ptr::copy(
                self.0.data.add(src_offset as usize),
                self.0.data.add(dst_offset as usize),
                size as usize,
            )
        }
        Ok(())
    }

    pub(crate) fn init(
        &self,
        data_source: &[u8],
        src_offset: u32,
        dst_offset: u32,
        size: Option<u32>,
    ) -> Result<(), MemoryError> {
        unsafe {
            let size = size
                .map(|s| s as u64)
                .unwrap_or((data_source.len().saturating_sub(src_offset as usize)) as u64);
            if src_offset as u64 + size > data_source.len() as u64 {
                return Err(MemoryError::DataSourceOOB);
            }
            if dst_offset as u64 + size > self.0.size as u64 * WASM_PAGE_SIZE as u64 {
                return Err(MemoryError::MemoryInitOOB);
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
        if let Err(e) = self.init(&data_instance.init, src_offset, dst_offset, Some(size)) {
            return Err(RuntimeError::Msg(format!(
                "Failed to initialize memory: {}",
                e
            )));
        }
        Ok(())
    }
}

impl Drop for MemoryObject {
    fn drop(&mut self) {
        if unsafe {
            libc::munmap(
                self.0.data as *mut libc::c_void,
                WASM_RESERVED_MEMORY_SIZE as usize,
            )
        } != 0
        {
            log::error!(
                "Failed to unmap memory at 0x{:x}: {}",
                self.0.data as usize,
                Errno::last()
            );
        }
    }
}

pub(crate) struct MemoryStorage<'a>(pub(crate) &'a mut [MemoryObject]);

impl<'a> MemoryStorage<'a> {
    pub(crate) fn init_on_cluster(
        cluster: &'a Cluster,
        memories_meta: &[Memory],
        data_meta: &[Data],
        imports: &[RTMemoryImport],
        globals: &GlobalStorage,
    ) -> Result<&'a mut [MemoryObject], MemoryError> {
        let mut memories = Vec::with_capacity(1);

        let mut imports_iter = imports.iter();
        let memories_meta_iter = memories_meta.iter().map(|m| {
            if m.import {
                imports_iter.next().unwrap().limits
            } else {
                m.limits
            }
        });
        for limits in memories_meta_iter {
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
                return Err(MemoryError::AllocationFailure(Errno::last()));
            }

            if 0 != unsafe {
                libc::mprotect(
                    memory_ptr,
                    (limits.min * WASM_PAGE_SIZE) as usize,
                    libc::PROT_READ | libc::PROT_WRITE,
                )
            } {
                return Err(MemoryError::AllocationFailure(Errno::last()));
            }
            memories.push(MemoryObject::new(
                memory_ptr as *mut u8,
                limits.min,
                limits.max.unwrap_or(WASM_PAGE_LIMIT),
            ))
        }

        for data in data_meta {
            if let DataMode::Active { memory, offset } = &data.mode {
                if *memory != 0 {
                    return Err(MemoryError::MemoryIdxNotZero);
                }
                let memory = &memories[*memory as usize];
                let offset = match offset {
                    ConstantValue::V(Value::Number(Number::I32(offset))) => *offset,
                    ConstantValue::V(v) => return Err(MemoryError::InvalidOffsetType(v.r#type())),
                    ConstantValue::Global(idx) => {
                        unsafe { *globals.globals[*idx as usize].addr }.as_u32()
                    }
                    ConstantValue::FuncPtr(_) => {
                        unimplemented!()
                    }
                };
                memory.init(data.init.as_slice(), 0, offset, None)?;
                // TODO: drop data
            }
        }

        Ok(cluster.alloc_memories(memories))
    }
}

impl Index<usize> for MemoryStorage<'_> {
    type Output = MemoryObject;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

#[no_mangle]
extern "C" fn memory_grow(
    ctxt: &runtime_interface::ExecutionContext,
    memory_idx: usize,
    grow_by: u32,
) -> i32 {
    let memories = MemoryStorage(unsafe {
        slice::from_raw_parts_mut(ctxt.memories_ptr as *mut MemoryObject, ctxt.memories_len)
    });
    let memory = &mut memories.0[memory_idx];
    memory.grow(grow_by)
}

fn memory_fill_impl(
    ctxt: &mut runtime_interface::ExecutionContext,
    offset: u32,
    size: u32,
    value: u8,
) -> Result<(), MemoryError> {
    let memories = MemoryStorage(unsafe {
        slice::from_raw_parts_mut(ctxt.memories_ptr as *mut MemoryObject, ctxt.memories_len)
    });
    let memory = &memories.0[0];
    memory.fill(offset, size, value)
}

#[no_mangle]
extern "C" fn memory_fill(
    ctxt: &mut runtime_interface::ExecutionContext,
    memory_idx: MemIdx,
    offset: u32,
    size: u32,
    value: u8,
) {
    let res = memory_fill_impl(ctxt, offset, size, value);
    trap_on_err(ctxt, res)
}

fn memory_copy_impl(
    ctxt: &mut runtime_interface::ExecutionContext,
    src_offset: u32,
    dst_offset: u32,
    size: u32,
) -> Result<(), MemoryError> {
    let memories = MemoryStorage(unsafe {
        slice::from_raw_parts_mut(ctxt.memories_ptr as *mut MemoryObject, ctxt.memories_len)
    });
    let memory = &memories.0[0];
    memory.copy(src_offset, dst_offset, size)
}

#[no_mangle]
extern "C" fn memory_copy(
    ctxt: &mut runtime_interface::ExecutionContext,
    memory_idx: MemIdx,
    src_offset: u32,
    dst_offset: u32,
    size: u32,
) {
    let res = memory_copy_impl(ctxt, src_offset, dst_offset, size);
    trap_on_err(ctxt, res)
}

fn memory_init_impl(
    ctxt: &mut runtime_interface::ExecutionContext,
    memory_idx: MemIdx,
    data_idx: DataIdx,
    src_offset: u32,
    dst_offset: u32,
    size: u32,
) -> Result<(), RuntimeError> {
    let memories = MemoryStorage(unsafe {
        slice::from_raw_parts_mut(ctxt.memories_ptr as *mut MemoryObject, ctxt.memories_len)
    });
    let memory = &memories.0[memory_idx as usize];
    memory.rt_init(
        ctxt.wasm_module.clone(),
        data_idx,
        src_offset,
        dst_offset,
        size,
    )
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
    let res = memory_init_impl(ctxt, memory_idx, data_idx, src_offset, dst_offset, size);
    trap_on_err(ctxt, res)
}

fn data_drop_impl(
    ctxt: &mut runtime_interface::ExecutionContext,
    data_idx: DataIdx,
) -> Result<(), MemoryError> {
    if data_idx as usize >= ctxt.wasm_module.datas.len() {
        return Err(MemoryError::DataIdxOOB);
    }
    // we don't remove elems
    Ok(())
}

#[no_mangle]
extern "C" fn data_drop(ctxt: &mut runtime_interface::ExecutionContext, data_idx: DataIdx) {}
