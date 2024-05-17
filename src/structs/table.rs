use crate::wasm_types::wasm_type::TableType;

#[derive(Debug, Clone)]
pub(crate) struct Table {
    pub(crate) r#type: TableType,
}
