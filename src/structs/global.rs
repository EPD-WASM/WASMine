use super::expression::Expression;
use wasm_types::module::GlobalType;

#[derive(Debug, Clone)]
pub(crate) struct Global {
    pub(crate) r#type: GlobalType,
    pub(crate) value: Expression,
    pub(crate) import: bool,
}
