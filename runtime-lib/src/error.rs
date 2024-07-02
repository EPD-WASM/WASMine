use nix::errno::Errno;
use thiserror::Error;
use wasm_types::ValType;

#[cfg(feature = "llvm")]
use llvm_gen::{ExecutionError, TranslationError};

#[cfg(feature = "interp")]
use interpreter::InterpreterError;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("Runtime Error: {0}")]
    Msg(String),
    #[error("Allocation Failure ({0})")]
    AllocationFailure(Errno),
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

    #[cfg(feature = "interp")]
    #[error("Interpreter error: {0}")]
    InterpreterError(#[from] InterpreterError),

    #[cfg(feature = "llvm")]
    #[error("LLVM translator error: {0}")]
    TranslationError(#[from] TranslationError),
    #[cfg(feature = "llvm")]
    #[error("LLVM execution error: {0}")]
    ExecutionError(#[from] ExecutionError),
}
