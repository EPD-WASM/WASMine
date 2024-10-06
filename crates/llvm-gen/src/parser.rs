use crate::{function_builder::LLVMFunctionBuilder, LLVMAdditionalResources};
use module::{objects::function::FunctionSource, Module as WasmModule, ModuleError};
use wasm_types::FuncIdx;

pub fn parse_wasm_functions(module: &WasmModule) -> Result<(), ModuleError> {
    log::info!("Loading functions using `llvm-gen`.");
    let artifact_registry = module.artifact_registry.read().unwrap();
    let llvm_resources = artifact_registry.get("llvm-module");
    if llvm_resources.is_none() {
        return Err(module::ModuleError::Msg(
            "LLVM resources not found in module. Parse module meta before running function parser."
                .to_string(),
        ));
    }
    let mut artifact_ref = llvm_resources.unwrap().write().unwrap();
    let llvm_resources = artifact_ref
        .downcast_mut::<LLVMAdditionalResources>()
        .unwrap();

    // skip if already parsed
    if llvm_resources.functions_parsed {
        log::info!("Functions already parsed by `llvm-gen`. Skipping.");
        return Ok(());
    }

    for func_idx in 0..module.meta.functions.len() {
        let function = &module.meta.functions[func_idx];
        match &function.source {
            FunctionSource::Import(_) => continue,
            FunctionSource::Wasm(function_unparsed) => {
                // function has unparsed mem and is not an import!
                let mut function_builder = LLVMFunctionBuilder::new(
                    llvm_resources.context.clone(),
                    func_idx as FuncIdx,
                    llvm_resources.module.clone(),
                    llvm_resources.functions.clone(),
                    &module.meta,
                );
                parser::FunctionLoader::default()
                    .parse_single_function(
                        &module.source,
                        func_idx as FuncIdx,
                        &function_unparsed,
                        &module.meta,
                        &mut function_builder,
                    )
                    .map_err(|e| module::ModuleError::Msg(e.to_string()))?;
                function_builder.finalize();
            }
        }
    }
    llvm_resources.functions_parsed = true;
    Ok(())
}
