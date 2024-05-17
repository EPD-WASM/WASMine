use thiserror::Error;

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
}

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Validation error: {0}")]
    Msg(String),
}
