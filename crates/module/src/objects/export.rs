use std::collections::HashMap;

use rkyv::{Deserialize, Serialize, Archive};
use wasm_types::{FuncIdx, GlobalIdx, MemIdx, Name, TableIdx};

#[derive(Debug, Clone)]
pub struct FuncExport {
    pub name: Name,
    pub idx: FuncIdx,
}

#[derive(Debug, Clone)]
pub struct TableExport {
    pub name: Name,
    pub idx: TableIdx,
}

#[derive(Debug, Clone)]
pub struct MemoryExport {
    pub name: Name,
    pub idx: MemIdx,
}

#[derive(Debug, Clone)]
pub struct GlobalExport {
    pub name: Name,
    pub idx: GlobalIdx,
}

#[derive(Debug, Clone)]
pub enum Export {
    Func(FuncExport),
    Table(TableExport),
    Mem(MemoryExport),
    Global(GlobalExport),
}

#[derive(Debug, Clone, Default, Archive, Deserialize, Serialize)]
pub struct WasmExports {
    pub functions: HashMap<String, FuncIdx>,
    pub functions_rev: HashMap<FuncIdx, String>,
    pub tables: HashMap<String, TableIdx>,
    pub memories: HashMap<String, MemIdx>,
    pub globals: HashMap<String, GlobalIdx>,
}

impl WasmExports {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn add_function_export(&mut self, e: FuncExport) {
        self.functions.insert(e.name.clone(), e.idx);
        self.functions_rev.insert(e.idx, e.name);
    }

    #[inline]
    pub fn add_table_export(&mut self, e: TableExport) {
        self.tables.insert(e.name, e.idx);
    }

    #[inline]
    pub fn add_memory_export(&mut self, e: MemoryExport) {
        self.memories.insert(e.name, e.idx);
    }

    #[inline]
    pub fn add_global_export(&mut self, e: GlobalExport) {
        self.globals.insert(e.name, e.idx);
    }

    pub fn find_function_name(&self, idx: FuncIdx) -> Option<&str> {
        self.functions_rev.get(&idx).map(String::as_str)
    }

    pub fn find_memory_idx(&self, name: &str) -> Option<MemIdx> {
        self.memories.get(name).copied()
    }

    pub fn find_function_idx(&self, name: &str) -> Option<FuncIdx> {
        self.functions.get(name).copied()
    }

    pub fn find_table_idx(&self, name: &str) -> Option<TableIdx> {
        self.tables.get(name).copied()
    }

    pub fn find_global_idx(&self, name: &str) -> Option<GlobalIdx> {
        self.globals.get(name).copied()
    }

    pub fn append(&mut self, other: Self) {
        self.functions.extend(other.functions);
        self.tables.extend(other.tables);
        self.memories.extend(other.memories);
        self.globals.extend(other.globals);
        self.functions_rev.extend(other.functions_rev);
    }

    pub fn functions(&self) -> impl Iterator<Item = (&String, &FuncIdx)> {
        self.functions.iter()
    }
}
