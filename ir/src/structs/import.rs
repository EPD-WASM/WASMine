use wasm_types::{ImportDesc, Name};

#[derive(Debug, Clone)]
pub struct Import {
    pub module: Name,
    pub name: Name,
    pub desc: ImportDesc,
}
