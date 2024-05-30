use wasm_types::{GlobalType, Limits, Name, TableType, TypeIdx};

#[derive(Debug, Clone)]
pub struct Import {
    pub module: Name,
    pub name: Name,
    pub desc: ImportDesc,
}

#[derive(Debug, Clone)]
pub enum ImportDesc {
    Func(TypeIdx),
    Table(TableType),
    Mem(Limits),
    Global(GlobalType),
}
