use wasm_types::module::TableType;

#[derive(Debug, Clone)]
pub(crate) struct Table {
    pub(crate) r#type: TableType,
}
