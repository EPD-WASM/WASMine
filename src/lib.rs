use module::objects::value::Value;
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

    let module = parser::Parser::parse_from_file(path)?;
    let module = Rc::new(module);
    engine.init(module.clone())?;

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
        Some(name) => match module.meta.exports.find_function_idx(&name) {
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
    use super::*;
    use llvm_gen::LLVMAdditionalResources;
    use resource_buffer::SourceFormat;
    use runtime_lib::ResourceBuffer;
    use std::rc::Rc;

    pub fn compile_internal(in_path: &Path, out_path: &Path) -> Result<(), RuntimeError> {
        if SourceFormat::from_path(in_path)? == SourceFormat::Cwasm {
            return Err(RuntimeError::Msg(
                "Cwasm files can't be compiled AGAIN... Please provide a wasm file.".to_owned(),
            ));
        }
        let module = ResourceBuffer::from_file(in_path)?;
        let module = llvm_gen::aot::parse_aot_module(module)?;
        let module = Rc::new(module);

        let (llvm_module, llvm_context) = {
            let artifacts_ref = module.artifact_registry.read().unwrap();
            let llvm_resources = artifacts_ref.get("llvm-module").unwrap().read().unwrap();
            let llvm_resources = llvm_resources
                .downcast_ref::<LLVMAdditionalResources>()
                .unwrap();
            (
                llvm_resources.module.clone(),
                llvm_resources.context.clone(),
            )
        };

        let mut executor = llvm_gen::JITExecutor::new(llvm_context)?;
        executor.add_module(llvm_module)?;

        let llvm_module_object_buf = executor.get_module_as_object_buffer(0)?;
        llvm_gen::aot::store_aot_module(&module, llvm_module_object_buf, out_path)?;
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
