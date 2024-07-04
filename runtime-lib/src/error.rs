use crate::{engine::EngineError, linker::LinkingError, module_instance::InstantiationError};
use thiserror::Error;
use wasm_types::ValType;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("Runtime Error: {0}")]
    Msg(String),
    #[error("Table Setup Error: {0}")]
    TableSetupError(String),
    #[error("Table Access Out of Bounds")]
    TableAccessOutOfBounds,
    #[error("Not implemented: {0}")]
    NotImplemented(String),
    #[error("Invalid import: {0}")]
    InvalidImport(String),
    #[error("Could not find entry point")]
    NoStartFunction,
    #[error("Invalid number of arguments. Required: {0}, provided: {1}")]
    ArgumentNumberMismatch(usize, usize),
    #[error("Invalid argument type. Expected: {0}, provided: {1}")]
    InvalidArgumentType(ValType, String),
    #[error("Trap: {0}")]
    Trap(String),

    #[error("Engine error: {0}")]
    EngineError(#[from] EngineError),

    #[error("Linking error: {0}")]
    LinkingError(#[from] LinkingError),

    #[error("Instantiation error: {0}")]
    InstatiationError(#[from] InstantiationError),
}
