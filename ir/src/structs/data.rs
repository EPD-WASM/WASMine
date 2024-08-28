use super::value::ConstantValue;
use bitcode::{Encode, Decode};
use wasm_types::MemIdx;

#[derive(Debug, Clone, PartialEq, Decode, Encode)]
pub enum DataMode {
    Active {
        memory: MemIdx,
        offset: ConstantValue,
    },
    Passive,
}

#[derive(Debug, Clone, Decode, Encode)]
pub struct Data {
    pub init: Vec<u8>,
    pub mode: DataMode,
}
