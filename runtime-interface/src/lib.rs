use wasm_types::FuncType;

pub trait ExecutionContext {
    extern "C" fn memory_grow(&self, memory_idx: usize, grow_by: u32) -> i32;
    extern "C" fn memory_fill(&self, memory_idx: usize, offset: usize, size: usize, value: u8);
    extern "C" fn memory_copy(
        &self,
        memory_idx: usize,
        src_offset: usize,
        dst_offset: usize,
        size: usize,
    );
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
