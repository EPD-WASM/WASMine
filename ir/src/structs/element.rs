use super::value::ConstantValue;
use wasm_types::{FuncIdx, RefType, TableIdx};

#[derive(Debug, Clone)]
pub enum ElementInit {
    Unresolved(Vec<FuncIdx>),
    Final(Vec<ConstantValue>),
}

#[derive(Debug, Clone)]
pub struct Element {
    pub type_: RefType,
    pub init: ElementInit,
    pub mode: ElemMode,
}

#[derive(Debug, Clone)]
pub enum ElemMode {
    Passive,
    Active {
        table: TableIdx,
        offset: ConstantValue,
    },
    Declarative,
}
