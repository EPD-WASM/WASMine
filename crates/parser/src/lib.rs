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

pub use ir::FunctionParser;
use module::{
    objects::module::{FunctionLoaderInterface, ModuleMetaLoaderInterface, ModuleStorerInterface},
    ModuleError, ModuleMetadata,
};
use resource_buffer::{ResourceBuffer, SourceFormat};
use rkyv::ser::{
    serializers::{
        AllocScratch, CompositeSerializer, FallbackScratch, HeapScratch, SharedSerializeMap,
        WriteSerializer,
    },
    Serializer,
};
use std::io::Write;
use std::{
    fs::File,
    io::{Seek, SeekFrom},
    path::Path,
};
use wasm_stream_reader::WasmBinaryReader;

#[derive(Debug, Default)]
pub struct ModuleMetaLoader;

impl ModuleMetaLoader {
    pub fn new() -> Self {
        Self::default()
    }

    fn parse_module_meta(
        &self,
        module: &mut ModuleMetadata,
        buffer: &ResourceBuffer,
    ) -> Result<(), ParserError> {
        log::info!("Loading module meta using `parser`");
        let mut instance = ModuleParser {
            module,
            is_complete: false,
            next_empty_function: 0,
        };
        let input = buffer.get()?;
        let mut reader = WasmBinaryReader::new(&input);
        match instance.parse_module(&mut reader) {
            Err(e) => Err(ParserError::PositionalError(Box::new(e), reader.pos)),
            _ => {
                #[cfg(debug_assertions)]
                {
                    // write parsed module to file as string
                    let mut f = std::fs::File::create("debug_output.parsed").unwrap();
                    f.write_all(module.to_string().as_bytes()).unwrap();
                }
                Ok(())
            }
        }
    }

    fn load_aot_module_meta(
        &self,
        module: &mut ModuleMetadata,
        buffer: &ResourceBuffer,
    ) -> Result<(), ParserError> {
        log::info!("Loading aot module meta using `parser`");
        let input = buffer.get()?;
        let wasm_module_buffer_size = u32::from_be_bytes(input[0..4].try_into().unwrap()) as usize;

        #[allow(never_type_fallback_flowing_into_unsafe)]
        let module_meta =
            unsafe { rkyv::from_bytes_unchecked(&input[8..8 + wasm_module_buffer_size]) }
                .map_err(|e| ParserError::Msg(format!("Failed to decode module metadata: {e}")))?;
        *module = module_meta;
        Ok(())
    }
}

impl ModuleMetaLoaderInterface for ModuleMetaLoader {
    fn load_module_meta(
        &self,
        module: &mut ModuleMetadata,
        buffer: &ResourceBuffer,
        _: &mut Vec<Box<dyn std::any::Any>>,
    ) -> Result<(), module::error::ModuleError> {
        match buffer.kind() {
            SourceFormat::Wasm => self.parse_module_meta(module, buffer),
            SourceFormat::Cwasm => self.load_aot_module_meta(module, buffer),
        }
        .map_err(|e| ModuleError::Msg(e.to_string()))
    }
}

#[derive(Debug, Default)]
pub struct FunctionLoader;

impl FunctionLoader {
    pub fn new() -> Self {
        Self::default()
    }

    fn load_wasm_functions_ir(
        &self,
        module: &mut ModuleMetadata,
        buffer: &ResourceBuffer,
    ) -> Result<(), ParserError> {
        log::info!("Loading functions using `parser` to ir");
        FunctionParser::parse_all_functions(module, buffer)
    }

    fn load_aot_functions(
        &self,
        module: &mut ModuleMetadata,
        buffer: &ResourceBuffer,
    ) -> Result<(), ParserError> {
        log::info!("Loading aot llvm functions using `parser`");
        let input = buffer.get()?;
        let llvm_memory_buffer_offset =
            u32::from_be_bytes(input[4..8].try_into().unwrap()) as usize;
        let llvm_memory_buffer = &input[llvm_memory_buffer_offset..];
        for function in module.functions.iter_mut() {
            function.add_precompiled_llvm(
                llvm_memory_buffer.as_ptr() as u64,
                input.len() - llvm_memory_buffer_offset,
            );
        }
        Ok(())
    }
}

impl FunctionLoaderInterface for FunctionLoader {
    fn parse_all_functions(
        &self,
        module: &mut ModuleMetadata,
        buffer: &ResourceBuffer,
        _: &mut Vec<Box<dyn std::any::Any>>,
    ) -> Result<(), ModuleError> {
        match buffer.kind() {
            SourceFormat::Wasm => self.load_wasm_functions_ir(module, buffer),
            SourceFormat::Cwasm => self.load_aot_functions(module, buffer),
        }
        .map_err(|e| ModuleError::Msg(e.to_string()))
    }
}

#[derive(Debug, Default)]
pub struct ModuleStorer;

impl ModuleStorerInterface for ModuleStorer {
    fn store(
        &self,
        module: &ModuleMetadata,
        llvm_memory_buffer: impl AsRef<[u8]>,
        output_path: impl AsRef<Path>,
    ) -> Result<(), ModuleError> {
        let mut out_file = File::create(output_path)?;
        out_file.seek(SeekFrom::Start(8))?;

        let mut serializer = CompositeSerializer::new(
            WriteSerializer::new(&mut out_file),
            <FallbackScratch<HeapScratch<1024>, AllocScratch>>::default(),
            SharedSerializeMap::default(),
        );
        debug_assert_eq!(serializer.pos(), 0);
        serializer.serialize_value(module).unwrap();
        let wasm_module_serialized_size = serializer.pos();

        let llvm_obj_offset =
            (2 * std::mem::size_of::<u32>() + wasm_module_serialized_size).next_multiple_of(2);

        out_file.seek(SeekFrom::Start(0))?;
        out_file.write_all(&u32::to_be_bytes(wasm_module_serialized_size as u32))?;
        out_file.write_all(&u32::to_be_bytes(llvm_obj_offset as u32))?;

        out_file.seek(SeekFrom::Start(llvm_obj_offset as u64))?;
        out_file.write_all(llvm_memory_buffer.as_ref())?;
        Ok(())
    }
}
