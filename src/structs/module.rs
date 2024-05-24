use super::{
    data::Data, element::Element, export::Export, global::Global, import::Import, memory::Memory,
    table::Table,
};
use crate::ir::function::Function;
use wasm_types::{FuncIdx, FuncType};

/// A WebAssembly module
#[derive(Debug, Clone, Default)]
pub struct Module {
    /// Wasm functions (code)
    pub(crate) functions: Vec<Function>,

    /// Wasm function tables (icall lookup)
    pub(crate) tables: Vec<Table>,
    /// Wasm elements (wasm table initialization)
    pub(crate) elements: Vec<Element>,

    /// Wasm memories (heaps)
    pub(crate) memories: Vec<Memory>,

    /// Wasm globals
    pub(crate) globals: Vec<Global>,

    /// Wasm data segments
    pub(crate) datas: Vec<Data>,

    /// Wasm number of data segments (if section is present)
    pub(crate) datacount: Option<u32>,

    /// Wasm function signatures
    pub(crate) function_types: Vec<FuncType>,

    /// Wasm start function
    pub(crate) entry_point: Option<FuncIdx>,

    /// Wasm imports
    pub(crate) imports: Vec<Import>,

    /// Wasm exports
    pub(crate) exports: Vec<Export>,
}
