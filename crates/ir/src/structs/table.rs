use bitcode::{Decode, Encode};
use wasm_types::TableType;

#[derive(Debug, Clone, Decode, Encode)]
pub struct Table {
    pub r#type: TableType,
    pub import: bool,
}
