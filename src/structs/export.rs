use crate::wasm_types::wasm_type::{FuncIdx, GlobalIdx, MemIdx, Name, TableIdx};

#[derive(Debug, Clone)]
pub(crate) struct Export {
    pub(crate) name: Name,
    pub(crate) desc: ExportDesc,
}

#[derive(Debug, Clone)]
pub(crate) enum ExportDesc {
    Func(FuncIdx),
    Table(TableIdx),
    Mem(MemIdx),
    Global(GlobalIdx),
}
