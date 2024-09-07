use ir::structs::value::Value;
use loader::SourceFormat;
use runtime_lib::{Cluster, Config, Engine, Linker, RuntimeError};
use std::{path::Path, rc::Rc};
use utils::parse_input_params_for_function;
use wasi::{PreopenDirInheritPerms, PreopenDirPerms, WasiContextBuilder};

mod cli;
mod utils;

pub use cli::main;

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
        let mut wasi_ctxt_builder = WasiContextBuilder::new();
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
    func.call(&function_args)
}

pub fn run(path: &Path, config: Config, engine: Engine, function_args: Vec<String>) -> u8 {
    match run_internal(path, config, engine, function_args) {
        Ok(return_values) => {
            log::info!(
                "Result: [{}]",
                return_values
                    .iter()
                    .map(ToString::to_string)
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
    use std::rc::Rc;

    use loader::CwasmLoader;

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
