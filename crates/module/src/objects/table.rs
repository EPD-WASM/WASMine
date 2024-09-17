use rkyv::{Deserialize, Serialize, Archive};
use wasm_types::TableType;

#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub struct Table {
    pub r#type: TableType,
    pub import: bool,
}
