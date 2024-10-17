use rkyv::{Archive, Deserialize, Serialize};
use wasm_types::Limits;

#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub struct Memory {
    pub limits: Limits,
    pub import: bool,
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub struct MemArg {
    pub offset: u32,
    pub align: u32,
}
