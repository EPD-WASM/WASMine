use serde::{Deserialize, Serialize};
use wasm_types::{ImportDesc, Name};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Import {
    pub module: Name,
    pub name: Name,
    pub desc: ImportDesc,
}
