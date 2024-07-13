use ir::{structs::expression::ConstantExpressionError, DecodingError};
use thiserror::Error;
use wasm_types::MemIdx;

#[derive(Debug, Error)]
pub enum ParserError {
    #[error("Parser error: {0}")]
    Msg(String),
    #[error("Parser error bevor byte 0x{1:x}: {0}")]
    PositionalError(Box<ParserError>, u32),
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
}

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Validation error: {0}")]
    Msg(String),
}
