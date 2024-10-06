use crate::{
    linker::LinkingError,
    objects::{
        engine::EngineError, functions::FunctionError, instance_handle::InstantiationError,
        memory::MemoryError, tables::TableError,
    },
};
use thiserror::Error;
use wasi::WasiError;
use wasm_types::ValType;

#[derive(Debug, Error, Default)]
pub enum RuntimeError {
    #[error("Runtime Error: {0}")]
    Msg(String),
    #[error("Not implemented: {0}")]
    NotImplemented(String),
    #[error("Invalid import: {0}")]
    InvalidImport(String),
    #[error("Could not find entry point")]
    NoStartFunction,
    #[error("Could not find function: {0}")]
    FunctionNotFound(String),
    #[error("Invalid number of arguments. Required: {0}, provided: {1}")]
    ArgumentNumberMismatch(usize, usize),
    #[error("Invalid argument type. Expected: {0}, provided: {1}")]
    InvalidArgumentType(ValType, String),
    #[error("Trap: {0}")]
    Trap(String),
    #[error("Stack exhausted")]
    Exhaustion,

    #[error("Engine error: {0}")]
    EngineError(#[from] EngineError),

    #[cfg(feature = "llvm")]
    #[error("LLVM execution error: {0}")]
    LLVMExecutionError(#[from] llvm_gen::ExecutionError),

    #[cfg(feature = "llvm")]
    #[error("LLVM aot error: {0}")]
    LLVMAotError(#[from] llvm_gen::aot::AOTError),

    #[cfg(feature = "llvm")]
    #[error("LLVM translation error: {0}")]
    LLVMTranslationError(#[from] llvm_gen::TranslationError),

    #[error("Linking error: {0}")]
    LinkingError(#[from] LinkingError),

    #[error("Instantiation error: {0}")]
    InstatiationError(#[from] InstantiationError),

    #[error("Table error: {0}")]
    TableError(#[from] TableError),

    #[error("Memory error: {0}")]
    MemoryError(#[from] MemoryError),

    #[error("Function error: {0}")]
    FunctionError(#[from] FunctionError),

    #[error("Wasi error: {0}")]
    WasiError(#[from] WasiError),

    #[error("ResourceBuffer error: {0}")]
    ResourceBufferError(#[from] resource_buffer::ResourceBufferError),

    #[error("Parser error: {0}")]
    ParserError(#[from] parser::ParserError),

    #[error("Module error: {0}")]
    ModuleError(#[from] module::ModuleError),

    #[default]
    #[error("No error.")]
    None,
}
