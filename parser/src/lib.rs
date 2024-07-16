mod context;
pub mod error;
mod function_builder;
pub(crate) mod instructions;
mod opcode_tbl;
pub(crate) mod parsable;
mod parse_basic_blocks;
#[allow(clippy::module_inception)]
pub mod parser;
mod stack;

pub(crate) mod wasm_stream_reader;

pub(crate) type ParseResult = Result<(), ParserError>;

pub use error::{ParserError, ValidationError};
pub use parser::Parser;
