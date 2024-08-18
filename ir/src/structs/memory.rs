use serde::{Deserialize, Serialize};
use wasm_types::Limits;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub limits: Limits,
    pub import: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemArg {
    pub offset: u32,
    pub align: u32,
}
