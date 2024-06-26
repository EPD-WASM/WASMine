use super::value::Value;
use wasm_types::{FuncIdx, RefType, TableIdx};

#[derive(Debug, Clone)]
pub enum ElementInit {
    Unresolved(Vec<FuncIdx>),
    Final(Vec<Value>),
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
    Active { table: TableIdx, offset: Value },
    Declarative,
}
