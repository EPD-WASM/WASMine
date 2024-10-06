use crate::error::ModuleError;

use super::{
    data::Data, element::Element, export::WasmExports, function::Function, global::Global,
    import::Import, memory::Memory, table::Table,
};
use resource_buffer::ResourceBuffer;
use rkyv::{Archive, Deserialize, Serialize};
use std::{any::Any, collections::HashMap, fmt::Debug, sync::RwLock};
use wasm_types::{FuncIdx, FuncType};

#[derive(Default, Debug, Deserialize, Serialize, Archive)]
pub struct ModuleMetadata {
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

    /// Wasm functions
    pub functions: Vec<Function>,
}

/// WebAssembly module metadata
pub struct Module {
    /// Module metadata
    pub meta: ModuleMetadata,

    /// Module data source
    ///
    /// Note: This typically needs to be kept alive, because sources are memory-mapped.
    pub source: ResourceBuffer,

    /// Additional resources
    ///
    /// e.g.:
    ///   - corresponding llvm module reference
    ///   - corresponding llvm object file
    ///   - parsed IR
    pub artifact_registry: RwLock<HashMap<String, RwLock<Box<dyn Any>>>>,
}

pub trait FunctionLoaderInterface {
    fn parse_all_functions(&self, module: &Module) -> Result<(), ModuleError>;
}

impl Module {
    pub fn load_all_functions(
        &self,
        loader: impl FunctionLoaderInterface,
    ) -> Result<(), ModuleError> {
        loader.parse_all_functions(&self)
    }
}
