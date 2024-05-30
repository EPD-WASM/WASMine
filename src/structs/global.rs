use super::{expression::ConstantExpression, value::Value};
use wasm_types::{module::GlobalType, NumType};

#[derive(Debug, Clone)]
pub(crate) struct Global {
    pub(crate) r#type: GlobalType,
    // the appropriate type transmuted to a u64
    pub(crate) init: Value,
    pub(crate) import: bool,
}
