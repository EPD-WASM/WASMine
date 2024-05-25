pub(crate) mod basic_block;
mod decoder;
mod encoder;
pub(crate) mod function;

// formatting is not production-save (lots of unwraps)
#[cfg(debug_assertions)]
pub(crate) mod fmt;

pub(crate) use decoder::{DecodingError, InstructionDecoder};
pub(crate) use encoder::InstructionEncoder;


#[derive(Clone, Default, Debug)]
pub(crate) struct IR {
    pub(crate) functions: Vec<function::Function>,
}
