use wasm_types::{module::Name, FuncIdx, GlobalIdx, MemIdx, TableIdx};

#[derive(Debug, Clone)]
pub struct Export {
    pub name: Name,
    pub desc: ExportDesc,
}

#[derive(Debug, Clone)]
pub enum ExportDesc {
    Func(FuncIdx),
    Table(TableIdx),
    Mem(MemIdx),
    Global(GlobalIdx),
}
