use super::value::Value;
use wasm_types::MemIdx;

#[derive(Debug, Clone, PartialEq)]
pub enum DataMode {
    Active { memory: MemIdx, offset: Value },
    Passive,
}

#[derive(Debug, Clone)]
pub struct Data {
    pub init: Vec<u8>,
    pub mode: DataMode,
}
