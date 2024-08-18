use serde::{Deserialize, Serialize};
use wasm_types::TableType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    pub r#type: TableType,
    pub import: bool,
}
