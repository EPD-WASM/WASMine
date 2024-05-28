use super::{
    expression::ConstantExpression,
    value::{Reference, Value},
};
use wasm_types::{FuncIdx, RefType, TableIdx};

#[derive(Debug, Clone)]
pub(crate) enum ElementInit {
    Unresolved(Vec<FuncIdx>),
    Final(Vec<Value>),
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
    Active { table: TableIdx, offset: Value },
    Declarative,
}
