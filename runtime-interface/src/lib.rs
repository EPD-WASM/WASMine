use ir::structs::{module::Module as WasmModule, value::ValueRaw};
use std::{ffi, ptr::NonNull, rc::Rc};
use wasm_types::{DataIdx, ElemIdx, MemIdx, TableIdx, TypeIdx};

pub type RawFunctionPtr = NonNull<core::ffi::c_void>;

/// The only top level datastructure always available to the executing WASM code
#[derive(Debug)]
#[repr(C)]
pub struct ExecutionContext {
    // runtime-resource slices
    pub tables_ptr: *mut ffi::c_void,
    pub tables_len: usize,

    pub globals_ptr: *mut GlobalStorage,
    pub globals_len: usize,

    pub memories_ptr: *mut MemoryInstance,
    pub memories_len: usize,

    pub trap_msg: Option<String>,

    pub wasm_module: Rc<WasmModule>,
    pub engine: *mut ffi::c_void,

    /// number of current recursion levels, used to prevent stack overflowing
    pub recursion_size: u32,

    // for debugging
    pub id: u32,
}

#[derive(Clone, Debug)]
#[repr(C)]
pub struct MemoryInstance {
    pub data: *mut u8,
    pub size: u32,
    pub max_size: u32,
}

#[derive(Clone)]
pub struct GlobalStorage {
    pub storage: *mut u8,
    pub globals: Vec<GlobalInstance>,
}

#[derive(Clone)]
pub struct GlobalInstance {
    pub addr: *mut ValueRaw,
}

// Careful! These function symbols are not mangled and pollute the global namespace!
// We only declare them here as a kind of interface (rust doesn't support native interfaces yet :/)
// we are not accessing all members of the execution context from the WASM side
// => not all members must be repr(C)
//
// u64 is enough to represent table values since we only ever receive reference types there
#[allow(improper_ctypes)]
extern "C" {
    pub fn memory_grow(ctxt: &mut ExecutionContext, memory_idx: usize, grow_by: u32) -> i32;
    pub fn memory_fill(
        ctxt: &mut ExecutionContext,
        memory_idx: MemIdx,
        offset: u32,
        size: u32,
        value: u8,
    );
    pub fn memory_copy(
        ctxt: &mut ExecutionContext,
        memory_idx: MemIdx,
        src_offset: u32,
        dst_offset: u32,
        size: u32,
    );
    pub fn memory_init(
        ctxt: &mut ExecutionContext,
        memory_idx: MemIdx,
        data_idx: DataIdx,
        src_offset: u32,
        dst_offset: u32,
        size: u32,
    );
    pub fn data_drop(ctxt: &mut ExecutionContext, data_idx: DataIdx);
    pub fn indirect_call(
        ctxt: &mut ExecutionContext,
        table_idx: TableIdx,
        type_idx: TypeIdx,
        entry_idx: u32,
    ) -> RawFunctionPtr;
    pub fn table_set(ctxt: &mut ExecutionContext, table_idx: usize, value: u64, idx: u32);
    pub fn table_get(ctxt: &mut ExecutionContext, table_idx: usize, idx: u32) -> u64;
    pub fn table_grow(
        ctxt: &mut ExecutionContext,
        table_idx: TableIdx,
        size: u32,
        value_to_fill: u64,
    ) -> u32;
    pub fn table_size(ctxt: &mut ExecutionContext, table_idx: usize) -> u32;
    pub fn table_fill(
        ctxt: &mut ExecutionContext,
        table_idx: usize,
        start: u32,
        len: u32,
        value: u64,
    );
    pub fn table_copy(
        ctxt: &mut ExecutionContext,
        src_table_idx: TableIdx,
        dst_table_idx: TableIdx,
        src_start: u32,
        dst_start: u32,
        len: u32,
    );
    pub fn table_init(
        ctxt: &mut ExecutionContext,
        table_idx: TableIdx,
        elem_idx: ElemIdx,
        src_offset: u32,
        dst_offset: u32,
        len: u32,
    );
    pub fn elem_drop(ctxt: &mut ExecutionContext, elem_idx: ElemIdx);
}
