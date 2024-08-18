use super::value::ConstantValue;
use serde::{Deserialize, Serialize};
use wasm_types::{FuncIdx, RefType, TableIdx};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ElementInit {
    Unresolved(Vec<FuncIdx>),
    Final(Vec<ConstantValue>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Element {
    pub type_: RefType,
    pub init: ElementInit,
    pub mode: ElemMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ElemMode {
    Passive,
    Active {
        table: TableIdx,
        offset: ConstantValue,
    },
    Declarative,
}
