use bitcode::{Decode, Encode};
use wasm_types::Limits;

#[derive(Debug, Clone, Decode, Encode)]
pub struct Memory {
    pub limits: Limits,
    pub import: bool,
}

#[derive(Debug, Clone, Decode, Encode)]
pub struct MemArg {
    pub offset: u32,
    pub align: u32,
}
