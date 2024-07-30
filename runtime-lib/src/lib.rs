#![allow(dead_code)]
#![allow(unused_variables)]

#[cfg(feature = "asm")]
compile_error!("The \"asm\" engine is not available yet.");

#[cfg(not(any(feature = "llvm", feature = "interp", feature = "asm")))]
compile_error!("You need to enable at least one execution backend!");

use ir::{
    structs::value::{Number, Value},
    utils::numeric_transmutes::{Bit32, Bit64},
};
use std::{path, rc::Rc};
use wasm_types::{FuncType, NumType, ValType};

mod cluster;
mod engine;
mod error;
mod execution_context;
mod func;
mod globals;
mod instance_handle;
mod linker;
mod memory;
mod segmented_list;
mod signals;
mod tables;
mod types;
mod utils;
mod wasi;

pub use cluster::Cluster;
pub use engine::Engine;
pub use error::RuntimeError;
pub use instance_handle::InstanceHandle;
pub use linker::{BoundLinker, Linker};

// reexports
pub use ir::structs::module::Module as WasmModule;
pub use loader::Loader;
pub use parser::{Parser, ParserError, ValidationError};

pub const WASM_PAGE_SIZE: u32 = 2_u32.pow(16);
// maximum amount of wasm pages
pub const WASM_PAGE_LIMIT: u32 = 2_u32.pow(16);
// maximal address accessible from 32-bit wasm code
pub const WASM_MAX_ADDRESS: u64 = 2_u64.pow(33) + 15;
// least amount of reserved intl pages to encorporate max wasm address
pub const WASM_RESERVED_MEMORY_SIZE: u64 = WASM_MAX_ADDRESS.next_multiple_of(INTL_PAGE_SIZE as u64);
// x86 small page size = 4KiB
pub const INTL_PAGE_SIZE: u32 = 2_u32.pow(12);

fn parse_input_params_for_function(function_type: &FuncType) -> Result<Vec<Value>, RuntimeError> {
    let args = std::env::args().skip(2);
    let num_args = args.len();
    if num_args != function_type.0.len() {
        return Err(RuntimeError::ArgumentNumberMismatch(
            function_type.0.len(),
            num_args,
        ));
    }
    let mut values = Vec::new();
    for (param_type, arg) in function_type.0.iter().zip(args) {
        let value = match param_type {
            ValType::Number(NumType::F32) => {
                Value::Number(Number::F32(arg.parse().map_err(|_| {
                    RuntimeError::InvalidArgumentType(param_type.to_owned(), arg)
                })?))
            }
            ValType::Number(NumType::F64) => {
                Value::Number(Number::F64(arg.parse().map_err(|_| {
                    RuntimeError::InvalidArgumentType(param_type.to_owned(), arg)
                })?))
            }
            ValType::Number(NumType::I32) => {
                let value: u32 = arg.parse::<u32>().or_else(|_| {
                    arg.parse::<i32>()
                        .map_err(|_| RuntimeError::InvalidArgumentType(param_type.to_owned(), arg))
                        .map(|i| i.trans_u32())
                })?;
                Value::Number(Number::I32(value))
            }
            ValType::Number(NumType::I64) => {
                let value: u64 = arg.parse::<u64>().or_else(|_| {
                    arg.parse::<i64>()
                        .map_err(|_| RuntimeError::InvalidArgumentType(param_type.to_owned(), arg))
                        .map(|i| i.trans_u64())
                })?;
                Value::Number(Number::I64(value))
            }
            _ => {
                return Err(RuntimeError::InvalidArgumentType(
                    param_type.to_owned(),
                    arg,
                ))
            }
        };
        values.push(value);
    }
    Ok(values)
}

fn run_internal(path: &str) -> Result<Vec<Value>, RuntimeError> {
    let path = path::Path::new(path);
    let loader = loader::Loader::from_file(path);
    let parser = parser::Parser::default();
    let module = Rc::new(parser.parse(loader).unwrap());

    let mut engine;

    #[cfg(feature = "interp")]
    {
        engine = Engine::interpreter()?;
    }
    #[cfg(all(not(feature = "interp"), feature = "llvm"))]
    {
        engine = Engine::llvm()?;
    }

    engine.init(module.clone())?;

    let cluster = Cluster::new();
    let mut linker = Linker::new();
    linker.link_wasi();

    let linker = linker.bind_to(&cluster);
    let mut module_handle = linker.instantiate_and_link(module.clone(), engine)?;
    module_handle.run_by_name(
        "_start",
        parse_input_params_for_function(
            module_handle
                .get_function_type_from_func_idx(module_handle.query_start_function().unwrap()),
        )?,
    )
}

pub fn run(path: &str) -> u8 {
    match run_internal(path) {
        Ok(return_values) => {
            for v in return_values {
                log::info!("Result: {}", v);
            }
            0
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            1
        }
    }
}
