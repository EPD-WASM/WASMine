mod abstraction;
pub mod aot;
mod error;
mod function_builder;
mod instructions;
mod jit_executor;
mod parser;
mod runtime_adapter;
mod translator;
mod util;

pub use abstraction::context::Context;
pub use error::*;
pub use jit_executor::JITExecutor;
pub use translator::Translator;

use abstraction::{function::Function, module::Module};
use module::{objects::module::FunctionLoaderInterface, Module as WasmModule};
use resource_buffer::SourceFormat;
use std::{cell::RefCell, rc::Rc};

pub(crate) struct LLVMAdditionalResources {
    pub(crate) module: Rc<Module>,
    functions: Rc<RefCell<Vec<Function>>>,
    pub(crate) context: Rc<Context>,
    functions_parsed: bool,
}

#[derive(Debug, Default)]
pub struct FunctionLoader;

impl FunctionLoader {}

impl FunctionLoaderInterface for FunctionLoader {
    fn parse_all_functions(&self, module: &WasmModule) -> Result<(), module::ModuleError> {
        match module.source.kind() {
            SourceFormat::Wasm => parser::parse_wasm_functions(module),
            SourceFormat::Cwasm => aot::parse_aot_functions(module)
                .map_err(|e| module::ModuleError::Msg(format!("Error parsing AOT functions: {e}"))),
        }
    }
}
