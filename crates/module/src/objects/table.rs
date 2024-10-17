use rkyv::{Archive, Deserialize, Serialize};
use wasm_types::TableType;

#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub struct Table {
    pub r#type: TableType,
    pub import: bool,
}
