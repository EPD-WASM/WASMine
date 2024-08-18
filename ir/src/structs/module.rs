use super::{
    data::Data, element::Element, export::WasmExports, global::Global, import::Import,
    memory::Memory, table::Table,
};
use crate::IR;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use wasm_types::{FuncIdx, FuncType};

/// A WebAssembly module
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Module {
    /// Wasm functions (code)
    pub ir: IR,

    /// Wasm function tables (icall lookup)
    pub tables: Vec<Table>,
    /// Wasm elements (wasm table initialization)
    pub elements: Vec<Element>,

    /// Wasm memories (heaps)
    pub memories: Vec<Memory>,

    /// Wasm globals
    pub globals: Vec<Global>,

    /// Wasm data segments
    pub datas: Vec<Data>,

    /// Wasm number of data segments (if section is present)
    pub datacount: Option<u32>,

    /// Wasm function signatures
    pub function_types: Vec<FuncType>,

    /// Wasm start function
    pub entry_point: Option<FuncIdx>,

    /// Wasm imports
    pub imports: Vec<Import>,

    /// Wasm exports
    pub exports: WasmExports,
}
