use std::ffi;

use wasm_types::FuncType;

/// The only top level datastructure always available to the executing WASM code
#[repr(C)]
pub struct ExecutionContext {
    pub id: u32,

    pub runtime: *mut ffi::c_void,

    /// number of current recursion levels, used to prevent stack overflowing
    pub recursion_size: u32,

    pub memories_ptr: *mut MemoryInstance,
    pub memories_len: usize,
    pub memories_cap: usize,
}

#[repr(C)]
pub struct MemoryInstance {
    pub data: *mut u8,
    pub size: u32,
}

#[derive(Clone)]
pub struct RTImport {
    pub name: &'static str,
    pub function_type: FuncType,
    pub callable: *const u8,
}

#[derive(Clone)]
pub struct GlobalStorage {
    pub storage: *mut u8,
    pub globals: Vec<GlobalInstance>,
}

#[derive(Clone)]
pub struct GlobalInstance {
    pub addr: *mut u64,
}

// we are not accessing all members of the execution context from the WASM side
// => not all members must be repr(C)
#[allow(improper_ctypes)]
extern "C" {
    pub fn memory_grow(ctxt: &ExecutionContext, memory_idx: usize, grow_by: u32) -> i32;
    pub fn memory_fill(
        ctxt: &ExecutionContext,
        memory_idx: usize,
        offset: usize,
        size: usize,
        value: u8,
    );
    pub fn memory_copy(
        ctxt: &ExecutionContext,
        memory_idx: usize,
        src_offset: usize,
        dst_offset: usize,
        size: usize,
    );
}
