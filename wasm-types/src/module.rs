use crate::*;

#[derive(Debug, Clone, Copy)]
pub struct TableType {
    pub ref_type: RefType,
    pub lim: Limits,
}

#[derive(Debug, Clone, Copy)]
pub enum GlobalType {
    Mut(ValType),
    Const(ValType),
}

#[derive(Debug, Clone)]
pub enum ExternType {
    Func(FuncType),
    Table(TableType),
    Mem(MemType),
    Global(GlobalType),
}

pub type Name = String;

#[derive(Debug, Clone, PartialEq)]
pub enum Section {
    Custom,
    Type,
    Import,
    Function,
    Table,
    Memory,
    Global,
    Export,
    Start,
    Element,
    Code,
    Data,
    DataCount,
}

pub type MemType = Limits;
