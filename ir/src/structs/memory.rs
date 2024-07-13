use wasm_types::Limits;

#[derive(Debug, Clone)]
pub struct Memory {
    pub limits: Limits,
    pub import: bool,
}

#[derive(Debug, Clone)]
pub struct MemArg {
    pub offset: u32,
    pub align: u32,
}
