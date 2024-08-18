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
use loader::SourceFormat;
pub use objects::engine::Engine;
pub use objects::instance_handle::InstanceHandle;
use std::{path::Path, rc::Rc};
use wasi::{PreopenDirInheritPerms, PreopenDirPerms};
use wasm_types::{FuncType, NumType, ValType};

mod cli;
mod cluster;
mod config;
mod error;
mod helper;
mod linker;
mod objects;
pub mod wasi;

pub use cli::main;
pub use cluster::{Cluster, ClusterConfig};
pub use error::RuntimeError;
pub use linker::{BoundLinker, Linker};

// reexports
pub use config::{Config, ConfigBuilder};
pub use ir::structs::module::Module as WasmModule;
pub use loader::{CwasmLoader, WasmLoader};
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

fn parse_input_params_for_function(
    args: Vec<String>,
    function_type: FuncType,
) -> Result<Vec<Value>, RuntimeError> {
    let num_args = args.len();
    if num_args != function_type.num_params() {
        return Err(RuntimeError::ArgumentNumberMismatch(
            function_type.num_params(),
            num_args,
        ));
    }
    let mut values = Vec::new();
    for (param_type, arg) in function_type.params_iter().zip(args) {
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

fn run_internal(
    path: &Path,
    config: Config,
    mut engine: Engine,
    function_args: Vec<String>,
) -> Result<Vec<Value>, RuntimeError> {
    log::debug!("run_internal: {:?}", config);

    let module = match SourceFormat::from_path(path)? {
        SourceFormat::Wasm => {
            let loader = loader::WasmLoader::from_file(path)?;
            let parser = parser::Parser::default();
            let module = Rc::new(parser.parse(loader).unwrap());
            engine.init(module.clone(), None)?;
            module
        }
        SourceFormat::Cwasm => {
            let loader = loader::CwasmLoader::from_file(path)?;
            let module = loader.wasm_module();
            engine.init(module.clone(), Some(&loader))?;
            module
        }
    };

    let start_function = config.start_function.clone();

    let cluster = Cluster::new(config.cluster_config);
    let linker = Linker::new();

    let linker = linker.bind_to(&cluster);
    let module_handle = if config.wasi_enabled {
        let mut wasi_ctxt_builder = wasi::WasiContextBuilder::new();
        wasi_ctxt_builder.args(config.wasi_args.clone());
        wasi_ctxt_builder.inherit_stdio();
        wasi_ctxt_builder.inherit_host_env();
        for (preopen_dir, path) in config.wasi_dirs.iter() {
            wasi_ctxt_builder.preopen_dir(
                preopen_dir,
                path.clone(),
                PreopenDirPerms::all(),
                PreopenDirInheritPerms::all(),
            )?;
        }
        let wasi_ctxt = wasi_ctxt_builder.finish();
        linker.instantiate_and_link_with_wasi(module.clone(), engine, wasi_ctxt)?
    } else {
        linker.instantiate_and_link(module.clone(), engine)?
    };

    let start_function = match start_function {
        Some(name) => match module.exports.find_function_idx(&name) {
            Some(idx) => idx,
            None => {
                log::error!(
                    "Could not find configured start function '{}' in wasm modules exports.",
                    name
                );
                return Err(RuntimeError::FunctionNotFound(name.to_owned()));
            }
        },
        None => match module_handle.query_start_function() {
            Ok(idx) => idx,
            Err(_) => {
                log::error!("Wasm module has no default start function. Please provide one explicitely via ");
                return Err(RuntimeError::FunctionNotFound(
                    "<start-function>".to_owned(),
                ));
            }
        },
    };
    let function_type = module_handle.get_function_type_from_func_idx(start_function);
    let function_args = parse_input_params_for_function(function_args, function_type)?;

    let func = module_handle.get_function_by_idx(start_function)?;
    func.call(&[])
}

pub fn run(path: &Path, config: Config, engine: Engine, function_args: Vec<String>) -> u8 {
    match run_internal(path, config, engine, function_args) {
        Ok(return_values) => {
            log::info!(
                "Result: [{}]",
                return_values
                    .iter()
                    .map(|v| format!("{}", v))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            0
        }
        Err(e) => {
            log::error!("Error: {}", e);
            1
        }
    }
}

#[cfg(feature = "llvm")]
mod c_wasm_compilation {
    use super::*;

    pub fn compile_internal(in_path: &Path, out_path: &Path) -> Result<(), RuntimeError> {
        if SourceFormat::from_path(in_path)? == SourceFormat::Cwasm {
            return Err(RuntimeError::Msg(
                "Cwasm files can't be compiled AGAIN... Please provide a wasm file.".to_owned(),
            ));
        }
        let loader = loader::WasmLoader::from_file(in_path)?;
        let parser = parser::Parser::default();
        let module = Rc::new(parser.parse(loader)?);

        let context = Rc::new(llvm_gen::Context::create());
        let mut executor = llvm_gen::JITExecutor::new(context.clone())?;
        let mut translator = llvm_gen::Translator::new(context.clone())?;

        let llvm_module = translator.translate_module(module.clone())?;
        executor.add_module(llvm_module)?;
        let llvm_module_object_buf = executor.get_module_as_object_buffer(0)?;
        CwasmLoader::write(out_path, module, llvm_module_object_buf)?;
        Ok(())
    }

    pub fn compile(in_path: &Path, out_path: &Path) -> u8 {
        match compile_internal(in_path, out_path) {
            Ok(_) => 0,
            Err(e) => {
                log::error!("Error: {}", e);
                1
            }
        }
    }
}
