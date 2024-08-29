#![allow(dead_code)]
#![allow(unused_variables)]

#[cfg(feature = "asm")]
compile_error!("The \"asm\" engine is not available yet.");

#[cfg(not(any(feature = "llvm", feature = "interp", feature = "asm")))]
compile_error!("You need to enable at least one execution backend!");

use helper::utils::parse_input_params_for_function;
use ir::structs::value::Value;
use loader::SourceFormat;
pub use objects::engine::Engine;
pub use objects::instance_handle::InstanceHandle;
use std::{path::Path, rc::Rc};
use wasi::{PreopenDirInheritPerms, PreopenDirPerms};

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
