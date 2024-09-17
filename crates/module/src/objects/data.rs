use super::value::ConstantValue;
use rkyv::{Archive, Deserialize, Serialize};
use wasm_types::MemIdx;

#[derive(Debug, Clone, PartialEq, Archive, Deserialize, Serialize)]
pub enum DataMode {
    Active {
        memory: MemIdx,
        offset: ConstantValue,
    },
    Passive,
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub struct Data {
    pub init: Vec<u8>,
    pub mode: DataMode,
}
