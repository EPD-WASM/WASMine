use super::value::ConstantValue;
use wasm_types::MemIdx;

#[derive(Debug, Clone, PartialEq)]
pub enum DataMode {
    Active {
        memory: MemIdx,
        offset: ConstantValue,
    },
    Passive,
}

#[derive(Debug, Clone)]
pub struct Data {
    pub init: Vec<u8>,
    pub mode: DataMode,
}
