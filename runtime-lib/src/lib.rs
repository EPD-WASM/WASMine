#![allow(dead_code)]
#![allow(unused_variables)]

use context::RTContext;
use error::RuntimeError;
use interpreter::InterpreterContext;
use ir::{
    structs::value::{Number, Value},
    utils::numeric_transmutes::{Bit32, Bit64},
};
use runtime::Runtime;
use std::path;
use wasm_types::{FuncType, NumType, ValType};
/* This runtime component library is responsible for:
    - Memory Management
    - WASI
    - Bootstrapping / Loading
    - Table Management
    - Data Management / Loading
*/
mod config;
mod context;
mod data;
mod error;
mod execution_context;
pub mod globals;
mod helpers;
mod memory;
mod runtime;
mod tables;
mod wasi;

pub const WASM_PAGE_SIZE: u32 = 2_u32.pow(16);
// maximum amount of wasm pages
pub const WASM_PAGE_LIMIT: u32 = 2_u32.pow(16);
// maximal address accessible from 32-bit wasm code
pub const WASM_MAX_ADDRESS: u32 = 2_u32 + 2_u32 + 15;
// least amount of reserved intl pages to encorporate max wasm address
pub const WASM_RESERVED_MEMORY_SIZE: usize =
    WASM_MAX_ADDRESS.next_multiple_of(INTL_PAGE_SIZE) as usize;
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

pub fn run(path: &str) -> u8 {
    let path = path::Path::new(path);
    let loader = loader::Loader::from_file(path);
    let parser = parser::Parser::default();
    let module = parser.parse(loader).unwrap();
    let context = match RTContext::new(module.clone()) {
        Ok(context) => context,
        Err(e) => {
            eprintln!("Error: {}", e);
            return 1;
        }
    };
    let start_function = match context.query_start_function() {
        Ok(start_function) => start_function,
        Err(e) => {
            eprintln!("Error: {}", e);
            return 1;
        }
    };
    let function_type = context.get_function_type(start_function);
    let input_params = match parse_input_params_for_function(function_type) {
        Ok(input_params) => input_params,
        Err(e) => {
            eprintln!("Error: {}", e);
            return 1;
        }
    };
    let runtime = Runtime::init(context);
    let execution_context = match unsafe { (*runtime).create_execution_context() } {
        Ok(execution_context) => execution_context,
        Err(e) => {
            eprintln!("Error: {}", e);
            return 1;
        }
    };
    let res = interpreter::Interpreter::new(InterpreterContext::new(module)).run(
        execution_context,
        start_function,
        unsafe { (*runtime).config.imports.clone() },
        unsafe { (*runtime).globals.clone() },
        input_params,
    );
    match res {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("Error: {}", RuntimeError::from(e));
            1
        }
    }
}
