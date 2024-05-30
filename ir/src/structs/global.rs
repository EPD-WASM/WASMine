use super::{expression::ConstantExpression, value::Value};
use wasm_types::{module::GlobalType, NumType};

#[derive(Debug, Clone)]
pub struct Global {
    pub r#type: GlobalType,
    // the appropriate type transmuted to a u64
    pub init: Value,
    pub import: bool,
}
