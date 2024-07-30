use crate::{
    engine::EngineError, func::FunctionError, instance_handle::InstantiationError,
    linker::LinkingError, memory::MemoryError, tables::TableError,
};
use thiserror::Error;
use wasm_types::ValType;

#[derive(Debug, Error)]
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
}
