pub mod error;
pub(crate) mod instructions;
pub(crate) mod ir;
#[allow(clippy::module_inception)]
pub mod module_parser;
pub(crate) mod parsable;
pub(crate) mod wasm_stream_reader;

pub(crate) type ParseResult = Result<(), ParserError>;

pub use crate::module_parser::ModuleParser;
pub use error::{ParserError, ValidationError};
pub use ir::context::Context;
pub use ir::function_builder::FunctionBuilderInterface;

use ir::FunctionParser;
use module::objects::function::FunctionUnparsed;
use module::{objects::module::FunctionLoaderInterface, Module, ModuleError, ModuleMetadata};
use resource_buffer::{ResourceBuffer, SourceFormat};
#[cfg(debug_assertions)]
use std::io::Write;
use std::path::Path;
use wasm_stream_reader::WasmBinaryReader;
use wasm_types::FuncIdx;

#[derive(Default)]
pub struct Parser;

impl Parser {
    pub fn parse_from_file(input_path: impl AsRef<Path>) -> Result<Module, ParserError> {
        log::debug!(
            "Loading module meta using `parser` from path: {:?}",
            input_path.as_ref()
        );
        let buffer = ResourceBuffer::from_file(input_path)?;
        Self::parse(buffer)
    }

    pub fn parse_from_buf(buf: Vec<u8>) -> Result<Module, ParserError> {
        log::debug!("Loading module meta using `parser` from buffer");
        let buffer = ResourceBuffer::from_wasm_buf(buf);
        Self::parse(buffer)
    }

    pub fn parse(buffer: ResourceBuffer) -> Result<Module, ParserError> {
        let mut module = Module {
            meta: ModuleMetadata::default(),
            source: buffer,
            artifact_registry: Default::default(),
        };
        let mut instance = ModuleParser {
            module: &mut module.meta,
            is_complete: false,
            next_empty_function: 0,
        };
        let input = module.source.get();
        let mut reader = WasmBinaryReader::new(&input);
        match instance.parse_module(&mut reader) {
            Err(e) => Err(ParserError::PositionalError(Box::new(e), reader.pos)),
            _ => {
                #[cfg(debug_assertions)]
                {
                    // write parsed module to file as string
                    let mut f = std::fs::File::create("debug_output.parsed").unwrap();
                    f.write_all(module.meta.to_string().as_bytes()).unwrap();
                }
                Ok(module)
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct FunctionLoader;

impl FunctionLoader {
    pub fn new() -> Self {
        Self::default()
    }

    fn load_wasm_functions_ir(&self, module: &Module) -> Result<(), ParserError> {
        log::debug!("Loading functions using `parser` to ir");
        FunctionParser::parse_all_functions(module)
    }

    pub fn parse_single_function(
        &self,
        buffer: &ResourceBuffer,
        function_idx: FuncIdx,
        function_unparsed: &FunctionUnparsed,
        module: &ModuleMetadata,
        builder: &mut impl FunctionBuilderInterface,
    ) -> ParseResult {
        FunctionParser::parse_single_function(
            buffer,
            function_idx,
            function_unparsed,
            module,
            builder,
        )
    }
}

impl FunctionLoaderInterface for FunctionLoader {
    fn parse_all_functions(&self, module: &Module) -> Result<(), ModuleError> {
        match module.source.kind() {
            SourceFormat::Wasm => self.load_wasm_functions_ir(module),
            SourceFormat::Cwasm => {
                return Err(ModuleError::Msg(
                    "AOT execution is only supported via the llvm function parser.".to_string(),
                ));
            }
        }
        .map_err(|e| ModuleError::Msg(e.to_string()))
    }
}
