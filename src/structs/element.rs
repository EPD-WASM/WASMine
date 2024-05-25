use super::expression::Expression;
use wasm_types::{FuncIdx, RefType, TableIdx};

#[derive(Debug, Clone)]
pub(crate) enum ElementInit {
    Unresolved(Vec<FuncIdx>),
    Final(Vec<Expression>),
}

#[derive(Debug, Clone)]
pub(crate) struct Element {
    pub(crate) type_: RefType,
    pub(crate) init: ElementInit,
    pub(crate) mode: ElemMode,
}

#[derive(Debug, Clone)]
pub(crate) enum ElemMode {
    Passive,
    Active { table: TableIdx, offset: Expression },
    Declarative,
}
