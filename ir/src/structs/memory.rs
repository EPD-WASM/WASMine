use thiserror::Error;
use wasm_types::Limits;

#[derive(Error, Debug, Clone)]
pub enum MemoryError {
    #[error("Index {index} out of range for limits {limits:?}")]
    OutOfRangeError { index: usize, limits: Limits },
}

#[derive(Debug, Clone)]
pub struct Memory {
    pub limits: Limits,
}

#[derive(Debug, Clone)]
pub struct MemArg {
    pub offset: u32,
    pub align: u32,
}
