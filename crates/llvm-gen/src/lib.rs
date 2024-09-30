mod abstraction;
mod error;
mod function_builder;
mod instructions;
mod jit_executor;
mod runtime_adapter;
mod translator;
mod util;

pub use abstraction::context::Context;
pub use error::*;
pub use jit_executor::JITExecutor;
pub use translator::Translator;

use abstraction::{function::Function, module::Module};
use function_builder::LLVMFunctionBuilder;
use module::{
    objects::module::{FunctionLoaderInterface, ModuleMetaLoaderInterface},
    ModuleError, ModuleMetadata,
};
use resource_buffer::{ResourceBuffer, SourceFormat};
use std::{cell::RefCell, rc::Rc};
use wasm_types::FuncIdx;

pub struct LLVMAdditionalResources {
    pub module: Rc<Module>,
    functions: Rc<RefCell<Vec<Function>>>,
    context: Rc<Context>,
    functions_parsed: bool,
}

pub struct ModuleMetaLoader;
impl ModuleMetaLoaderInterface for ModuleMetaLoader {
    fn load_module_meta(
        &self,
        module: &mut module::ModuleMetadata,
        buffer: &ResourceBuffer,
        additional_resources: &mut Vec<Box<dyn std::any::Any>>,
    ) -> Result<(), module::ModuleError> {
        log::info!("Loading module meta using `llvm-gen`.");
        // only parse module meta if not already done
        if additional_resources
            .iter()
            .any(|r| r.downcast_ref::<LLVMAdditionalResources>().is_some())
        {
            log::info!("Module meta already parsed by `llvm-gen`. Skipping.");
            return Ok(());
        }

        // parse module meta using parser if necessary
        if module.is_empty() {
            log::info!("Module meta (apparently) not parsed yet. Parsing now using `parser::ModuleMetaLoader`.");
            parser::ModuleMetaLoader::default().load_module_meta(
                module,
                buffer,
                additional_resources,
            )?;
        }

        let llvm_context = Rc::new(Context::create());
        let (llvm_module, llvm_functions) =
            Translator::translate_module_meta(llvm_context.clone(), module)
                .map_err(|e| module::ModuleError::Msg(e.to_string()))?;

        let resources = LLVMAdditionalResources {
            module: llvm_module,
            context: llvm_context,
            functions: llvm_functions,
            functions_parsed: false,
        };
        // TODO: this is only experimental and of course a very bad idea
        additional_resources.push(Box::new(resources));

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct FunctionLoader;

impl FunctionLoader {
    fn load_wasm_functions(
        &self,
        wasm_module: &mut ModuleMetadata,
        buffer: &ResourceBuffer,
        additional_resources: &mut Vec<Box<dyn std::any::Any>>,
    ) -> Result<(), ModuleError> {
        log::info!("Loading functions using `llvm-gen`.");
        let llvm_resources = additional_resources
            .iter_mut()
            .find_map(|r| r.downcast_mut::<LLVMAdditionalResources>());
        if llvm_resources.is_none() {
            return Err(module::ModuleError::Msg(
                "LLVM resources not found in module. Parse module meta before running function parser.".to_string(),
            ));
        }
        let llvm_resources = llvm_resources.unwrap();

        // skip if already parsed
        if llvm_resources.functions_parsed {
            log::info!("Functions already parsed by `llvm-gen`. Skipping.");
            return Ok(());
        }

        for func_idx in 0..wasm_module.functions.len() {
            let function = &wasm_module.functions[func_idx];
            if function.get_import().is_some() {
                continue;
            }

            if function.get_unparsed_mem().is_none() {
                match function.get_ir() {
                    None => {
                        return Err(module::ModuleError::Msg(
                            "function missing raw representation AND ir cannot be compiled to LLVM"
                                .to_string(),
                        ))
                    }
                    Some(ir) => Translator::translate_single_function(
                        llvm_resources.context.clone(),
                        &wasm_module,
                        llvm_resources.module.clone(),
                        llvm_resources.functions.clone(),
                        ir,
                        func_idx,
                    )
                    .map_err(|e| module::ModuleError::Msg(e.to_string()))?,
                }
                continue;
            }
            // function has unparsed mem and is not an import!
            let mut function_builder = LLVMFunctionBuilder::new(
                llvm_resources.context.clone(),
                func_idx as u32,
                llvm_resources.module.clone(),
                llvm_resources.functions.clone(),
                &wasm_module,
            );
            parser::FunctionParser::parse_single_function(
                buffer,
                func_idx as FuncIdx,
                wasm_module,
                &mut function_builder,
            )
            .map_err(|e| module::ModuleError::Msg(e.to_string()))?;
            function_builder.finalize();
        }
        llvm_resources.functions_parsed = true;
        Ok(())
    }
}

impl FunctionLoaderInterface for FunctionLoader {
    fn parse_all_functions(
        &self,
        wasm_module: &mut module::ModuleMetadata,
        buffer: &ResourceBuffer,
        additional_resources: &mut Vec<Box<dyn std::any::Any>>,
    ) -> Result<(), module::ModuleError> {
        match buffer.kind() {
            SourceFormat::Wasm => {
                self.load_wasm_functions(wasm_module, buffer, additional_resources)
            }
            // redirect to parser if AOT functions are available
            // (has ability to load AOT LLVM object files)
            SourceFormat::Cwasm => parser::FunctionLoader.parse_all_functions(
                wasm_module,
                buffer,
                additional_resources,
            ),
        }
    }
}
