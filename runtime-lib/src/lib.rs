#![allow(dead_code)]
#![allow(unused_variables)]

#[cfg(feature = "asm")]
compile_error!("The \"asm\" engine is not available yet.");

#[cfg(not(any(feature = "llvm", feature = "interp", feature = "asm")))]
compile_error!("You need to enable at least one execution backend!");

use error::RuntimeError;
use ir::{
    structs::value::{Number, Value},
    utils::numeric_transmutes::{Bit32, Bit64},
};
use runtime_interface::{GlobalInstance, MemoryInstance, RawFunctionPtr};
use std::{path, rc::Rc};
use tables::TableInstance;
use wasm_types::{FuncType, GlobalType, Limits, NumType, TableType, ValType};

pub mod engine;
pub mod error;
mod execution_context;
pub mod globals;
mod helpers;
pub mod linker;
mod memory;
pub mod module_instance;
pub mod runtime;
mod tables;
mod wasi;

pub const WASM_PAGE_SIZE: u32 = 2_u32.pow(16);
// maximum amount of wasm pages
pub const WASM_PAGE_LIMIT: u32 = 2_u32.pow(16);
// maximal address accessible from 32-bit wasm code
pub const WASM_MAX_ADDRESS: u64 = 2_u64.pow(33) + 15;
// least amount of reserved intl pages to encorporate max wasm address
pub const WASM_RESERVED_MEMORY_SIZE: u64 = WASM_MAX_ADDRESS.next_multiple_of(INTL_PAGE_SIZE as u64);
// x86 small page size = 4KiB
pub const INTL_PAGE_SIZE: u32 = 2_u32.pow(12);

#[derive(Clone, Debug)]
pub struct RTFuncImport {
    pub name: String,
    pub function_type: FuncType,
    pub callable: RawFunctionPtr,
}

#[derive(Clone, Debug)]
pub struct RTMemoryImport {
    pub name: String,
    pub instance: MemoryInstance,
    pub limits: Limits,
}

#[derive(Clone)]
pub struct RTGlobalImport {
    pub name: String,
    pub instance: GlobalInstance,
    pub r#type: GlobalType,
}

#[derive(Clone)]
pub struct RTTableImport {
    pub name: String,
    pub instance: TableInstance,
    pub r#type: TableType,
}

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

    #[cfg(feature = "interp")]
    {
        todo!()
    }
    #[cfg(feature = "llvm")]
    {
        let linker = Linker::new();
        let engine = Engine::llvm()?;

        let mut module_instance = linker.link(module, engine)?;
        module_instance.run_by_name(
            "_start",
            parse_input_params_for_function(
                module_instance.get_function_type(module_instance.query_start_function().unwrap()),
            )?,
        )
    }
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
