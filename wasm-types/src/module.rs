use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Decode, Encode)]
pub struct TableType {
    pub ref_type: RefType,
    pub lim: Limits,
}

#[derive(Debug, Clone, Copy, PartialEq, Decode, Encode)]
pub enum GlobalType {
    Mut(ValType),
    Const(ValType),
}

#[derive(Debug, Clone, Decode, Encode)]
pub enum ExternType {
    Func(FuncType),
    Table(TableType),
    Mem(MemType),
    Global(GlobalType),
}

#[derive(Debug, Clone, Decode, Encode)]
pub enum ImportDesc {
    Func(TypeIdx),
    Table(TableType),
    Mem(Limits),
    Global((GlobalType, GlobalIdx)),
}

pub type Name = String;

#[derive(Debug, Clone, PartialEq, Decode, Encode)]
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
