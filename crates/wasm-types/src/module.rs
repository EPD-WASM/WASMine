use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Archive, Deserialize, Serialize)]
pub struct TableType {
    pub ref_type: RefType,
    pub lim: Limits,
}

#[derive(Debug, Clone, Copy, PartialEq, Archive, Deserialize, Serialize)]
pub enum GlobalType {
    Mut(ValType),
    Const(ValType),
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub enum ExternType {
    Func(FuncType),
    Table(TableType),
    Mem(MemType),
    Global(GlobalType),
}

#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub enum ImportDesc {
    Func(TypeIdx),
    Table(TableType),
    Mem(Limits),
    Global((GlobalType, GlobalIdx)),
}

pub type Name = String;

// non-custom sections must appear in module at least once in a certain order defined by the spec.
// This enum contains all sections with numbers assigned in the order they must appear in the module.
#[derive(Debug, Clone, PartialEq, Archive, Deserialize, Serialize)]
pub enum Section {
    Custom = 99,
    Type = 0,
    Import = 1,
    Function = 2,
    Table = 3,
    Memory = 4,
    Global = 5,
    Export = 6,
    Start = 7,
    Element = 8,
    DataCount = 9,
    Code = 10,
    Data = 11,
}

pub type MemType = Limits;
