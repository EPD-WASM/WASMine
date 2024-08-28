use bitcode::{Encode, Decode};
use wasm_types::{ImportDesc, Name};

#[derive(Debug, Clone, Decode, Encode)]
pub struct Import {
    pub module: Name,
    pub name: Name,
    pub desc: ImportDesc,
}
