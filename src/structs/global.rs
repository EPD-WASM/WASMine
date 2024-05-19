use super::expression::Expression;
use crate::wasm_types::wasm_type::GlobalType;

#[derive(Debug, Clone)]
pub(crate) struct Global {
    pub(crate) r#type: GlobalType,
    pub(crate) value: Expression,
    pub(crate) import: bool,
}