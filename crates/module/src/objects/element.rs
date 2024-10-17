use super::value::ConstantValue;
use rkyv::{Archive, Deserialize, Serialize};
use wasm_types::{FuncIdx, RefType, TableIdx};

#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub enum ElementInit {
    Unresolved(Vec<FuncIdx>),
    Final(Vec<ConstantValue>),
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub struct Element {
    pub type_: RefType,
    pub init: ElementInit,
    pub mode: ElemMode,
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub enum ElemMode {
    Passive,
    Active {
        table: TableIdx,
        offset: ConstantValue,
    },
    Declarative,
}
