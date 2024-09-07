use super::value::ConstantValue;
use bitcode::{Decode, Encode};
use wasm_types::{FuncIdx, RefType, TableIdx};

#[derive(Debug, Clone, Decode, Encode)]
pub enum ElementInit {
    Unresolved(Vec<FuncIdx>),
    Final(Vec<ConstantValue>),
}

#[derive(Debug, Clone, Decode, Encode)]
pub struct Element {
    pub type_: RefType,
    pub init: ElementInit,
    pub mode: ElemMode,
}

#[derive(Debug, Clone, Decode, Encode)]
pub enum ElemMode {
    Passive,
    Active {
        table: TableIdx,
        offset: ConstantValue,
    },
    Declarative,
}
