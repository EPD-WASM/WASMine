use super::value::ConstantValue;
use serde::{Deserialize, Serialize};
use wasm_types::{GlobalType, ValType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Global {
    pub r#type: GlobalType,
    // the appropriate type transmuted to a u64
    pub init: ConstantValue,
    pub import: bool,
}

impl Global {
    pub fn val_type(&self) -> ValType {
        match self.r#type {
            GlobalType::Mut(val_type) | GlobalType::Const(val_type) => val_type,
        }
    }
}
