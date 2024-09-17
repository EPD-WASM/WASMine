use module::{objects::expression::ConstantExpressionError, DecodingError};
use thiserror::Error;
use wasm_types::{FuncIdx, MemIdx};

#[derive(Debug, Error)]
pub enum ParserError {
    #[error("Parser error: {0}")]
    Msg(String),
    #[error("Parser error bevor byte 0x{1:x}: {0}")]
    PositionalError(Box<ParserError>, usize),
    #[error("Invalid opcode")]
    InvalidOpcode,
    #[error("Invalid instruction encoding")]
    InvalidEncoding,
    #[error("Invalid LEB128 encoding")]
    InvalidLEB128Encoding,
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Validation error: {0}")]
    ValidationError(#[from] ValidationError),
    #[error("Constant expression error: {0}")]
    ConstantExpressionError(#[from] ConstantExpressionError),
    #[error("Decoding error: {0}")]
    DecodingError(#[from] DecodingError),
    #[error("unknown memory {0}")]
    UnknownMemory(MemIdx),
    #[error("size minimum must not be greater than maximum")]
    LimitsMinimumGreaterThanMaximum,
    #[error("referenced start function does not exist")]
    StartFunctionDoesNotExist,
    #[error("alignment must not be larger than natural")]
    AlignmentLargerThanNatural,
    #[error("Loader error: {0}")]
    LoaderError(#[from] resource_buffer::ResourceBufferError),
    #[error("Unexepected EOF")]
    UnexpectedEOF,
    #[error("Missing function implementation for function {0}")]
    MissingFunctionImplementation(FuncIdx),
}

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Validation error: {0}")]
    Msg(String),
}
