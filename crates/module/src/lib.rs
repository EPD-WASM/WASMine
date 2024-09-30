pub mod error;
pub mod instructions;
pub mod objects;
pub mod utils;

// formatting is not production-save (lots of unwraps)
#[cfg(debug_assertions)]
pub mod fmt;

pub use instructions::basic_block;
pub use instructions::basic_block::{BasicBlock, BasicBlockID};
pub use instructions::decoder::{DecodingError, InstructionDecoder};
pub use instructions::encoder::InstructionEncoder;
pub use instructions::instruction_consumer::InstructionConsumer;

pub use error::ModuleError;
pub use objects::module::{Module, ModuleMetadata};
