use crate::wasm_types::wasm_type::{GlobalType, MemType, Name, TableType, TypeIdx};

#[derive(Debug, Clone)]
pub(crate) struct Import {
    pub(crate) module: Name,
    pub(crate) name: Name,
    pub(crate) desc: ImportDesc,
}

#[derive(Debug, Clone)]
pub(crate) enum ImportDesc {
    Func(TypeIdx),
    Table(TableType),
    Mem(MemType),
    Global(GlobalType),
}
