pub mod basic_block;
mod decoder;
mod encoder;
pub mod function;
pub mod instructions;
pub mod structs;
pub mod utils;

// formatting is not production-save (lots of unwraps)
#[cfg(debug_assertions)]
pub mod fmt;

pub use decoder::{DecodingError, InstructionDecoder};
pub use encoder::InstructionEncoder;

use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct IR {
    pub functions: Vec<function::Function>,
}
