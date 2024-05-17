use super::expression::Expression;
use crate::wasm_types::wasm_type::MemIdx;

#[derive(Debug, Clone)]
pub(crate) enum DataMode {
    Active { memory: MemIdx, offset: Expression },
    Passive,
}

#[derive(Debug, Clone)]
pub(crate) struct Data {
    pub(crate) init: Vec<u8>,
    pub(crate) mode: DataMode,
}
