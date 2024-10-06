#![allow(dead_code)]
#![allow(unused_variables)]

#[cfg(feature = "asm")]
compile_error!("The \"asm\" engine is not available yet.");

#[cfg(not(any(feature = "llvm", feature = "interp", feature = "asm")))]
compile_error!("You need to enable at least one execution backend!");

pub use objects::engine::Engine;
pub use objects::instance_handle::InstanceHandle;

mod cluster;
mod config;
mod error;
mod helper;
mod linker;
mod objects;
pub mod sugar;

pub use cluster::{Cluster, ClusterConfig};
pub use error::RuntimeError;
pub use linker::{BoundLinker, Linker};

// reexports
pub use config::{Config, ConfigBuilder};
pub use module::{objects::module::FunctionLoaderInterface, objects::module::Module as WasmModule};
pub use parser::{FunctionLoader, Parser, ParserError, ValidationError};
pub use resource_buffer::ResourceBuffer;

pub const WASM_PAGE_SIZE: u32 = 2_u32.pow(16);
// maximum amount of wasm pages
pub const WASM_PAGE_LIMIT: u32 = 2_u32.pow(16);
// maximal address accessible from 32-bit wasm code
pub const WASM_MAX_ADDRESS: u64 = 2_u64.pow(33) + 15;
// least amount of reserved intl pages to encorporate max wasm address
pub const WASM_RESERVED_MEMORY_SIZE: u64 = WASM_MAX_ADDRESS.next_multiple_of(INTL_PAGE_SIZE as u64);
// x86 small page size = 4KiB
pub const INTL_PAGE_SIZE: u32 = 2_u32.pow(12);
