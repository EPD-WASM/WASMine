use super::value::ConstantValue;
use serde::{Deserialize, Serialize};
use wasm_types::MemIdx;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DataMode {
    Active {
        memory: MemIdx,
        offset: ConstantValue,
    },
    Passive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Data {
    pub init: Vec<u8>,
    pub mode: DataMode,
}
