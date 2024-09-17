use crate::error::ModuleError;

use super::{
    data::Data, element::Element, export::WasmExports, function::Function, global::Global,
    import::Import, memory::Memory, table::Table,
};
use resource_buffer::ResourceBuffer;
use rkyv::{Archive, Deserialize, Serialize};
use std::{fmt::Debug, path::Path};
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
    ///
    /// Note: Functions internally countain a list of lazy loaded representations (ir, llvm, unparsed, etc.).
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
}

pub trait ModuleMetaLoaderInterface {
    fn load_module_meta(
        &self,
        m: &mut ModuleMetadata,
        b: &ResourceBuffer,
    ) -> Result<(), ModuleError>;
}

pub trait FunctionLoaderInterface {
    fn parse_all_functions(
        &self,
        m: &mut ModuleMetadata,
        b: &ResourceBuffer,
    ) -> Result<(), ModuleError>;
}

pub trait ModuleStorerInterface {
    fn store(
        &self,
        m: &ModuleMetadata,
        llvm_memory_buffer: impl AsRef<[u8]>,
        output_path: impl AsRef<Path>,
    ) -> Result<(), ModuleError>;
}

impl Module {
    /// Create a new module from a loader and a parser.
    ///
    /// Takes ownership of the loader.
    /// Parser is an injected dependency to keep coupling low.
    pub fn new(source: ResourceBuffer) -> Self {
        Self {
            meta: ModuleMetadata::default(),
            source,
        }
    }

    pub fn load_meta(&mut self, loader: impl ModuleMetaLoaderInterface) -> Result<(), ModuleError> {
        loader.load_module_meta(&mut self.meta, &self.source)
    }

    pub fn load_all_functions(
        &mut self,
        loader: impl FunctionLoaderInterface,
    ) -> Result<(), ModuleError> {
        loader.parse_all_functions(&mut self.meta, &self.source)
    }

    pub fn store(
        &self,
        storer: impl ModuleStorerInterface,
        llvm_memory_buffer: impl AsRef<[u8]>,
        output_path: impl AsRef<Path>,
    ) -> Result<(), ModuleError> {
        storer.store(&self.meta, llvm_memory_buffer, output_path)
    }
}
