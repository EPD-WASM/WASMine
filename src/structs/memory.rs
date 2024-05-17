use crate::wasm_types::wasm_type::MemType;

#[derive(Debug, Clone)]
pub(crate) struct Memory {
    pub(crate) r#type: MemType,
}

#[derive(Debug, Clone)]
pub(crate) struct MemArg {
    pub(crate) offset: u32,
    pub(crate) align: u32,
}
