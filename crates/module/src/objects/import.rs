use rkyv::{Archive, Deserialize, Serialize};
use wasm_types::{ImportDesc, Name};

#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub struct Import {
    pub module: Name,
    pub name: Name,
    pub desc: ImportDesc,
}
