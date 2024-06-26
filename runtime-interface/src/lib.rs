use std::ffi;

use wasm_types::{DataIdx, ElemIdx, FuncType, MemIdx, TableIdx, TypeIdx};

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
        memory_idx: MemIdx,
        offset: usize,
        size: usize,
        value: u8,
    );
    pub fn memory_copy(
        ctxt: &ExecutionContext,
        memory_idx: MemIdx,
        src_offset: usize,
        dst_offset: usize,
        size: usize,
    );
    pub fn memory_init(
        ctxt: &ExecutionContext,
        memory_idx: MemIdx,
        data_idx: DataIdx,
        src_offset: usize,
        dst_offset: usize,
        size: usize,
    );
    pub fn data_drop(ctxt: &ExecutionContext, data_idx: DataIdx);
    pub fn indirect_call(
        ctxt: &ExecutionContext,
        table_idx: TableIdx,
        type_idx: TypeIdx,
        entry_idx: usize,
    ) -> u64;
    pub fn table_set(ctxt: &ExecutionContext, table_idx: usize, value: u64);
    pub fn table_get(ctxt: &ExecutionContext, table_idx: usize) -> u64;
    pub fn table_grow(
        ctxt: &ExecutionContext,
        table_idx: TableIdx,
        size: u32,
        value_to_fill: u64,
    ) -> i32;
    pub fn table_size(ctxt: &ExecutionContext, table_idx: usize) -> u32;
    pub fn table_fill(ctxt: &ExecutionContext, table_idx: usize, start: u32, len: u32, value: u64);
    pub fn table_copy(
        ctxt: &ExecutionContext,
        src_table_idx: TableIdx,
        dst_table_idx: TableIdx,
        src_start: u32,
        dst_start: u32,
        len: u32,
    );
    pub fn table_init(
        ctxt: &ExecutionContext,
        table_idx: TableIdx,
        elem_idx: ElemIdx,
        src_offset: u32,
        dst_offset: u32,
        len: u32,
    );
    pub fn elem_drop(ctxt: &ExecutionContext, elem_idx: ElemIdx);
}
