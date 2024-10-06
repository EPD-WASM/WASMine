use std::path::Path;

use module::{objects::module::FunctionLoaderInterface, Module, ModuleError};
use resource_buffer::{ResourceBuffer, SourceFormat};

pub fn module_from_file(file: &Path) -> Result<Module, ModuleError> {
    let buf = ResourceBuffer::from_file(file)?;
    match buf.kind() {
        SourceFormat::Wasm => {
            let module = parser::Parser::parse(buf).map_err(|e| ModuleError::Msg(e.to_string()))?;
            Ok(module)
        }
        #[cfg(feature = "llvm")]
        SourceFormat::Cwasm => {
            let module =
                llvm_gen::aot::parse_aot_meta(buf).map_err(|e| ModuleError::Msg(e.to_string()))?;
            llvm_gen::FunctionLoader::default().parse_all_functions(&module)?;
            Ok(module)
        }
        #[cfg(not(feature = "llvm"))]
        SourceFormat::Cwasm => Err(ModuleError::Msg(
            "Cwasm files are not supported without the `llvm` feature.".to_string(),
        )),
    }
}

pub fn module_from_buf(buf: Vec<u8>) -> Result<Module, ModuleError> {
    let buf = ResourceBuffer::from_wasm_buf(buf);
    debug_assert_eq!(buf.kind(), SourceFormat::Wasm);
    match buf.kind() {
        SourceFormat::Wasm => {
            let module = parser::Parser::parse(buf).map_err(|e| ModuleError::Msg(e.to_string()))?;
            parser::FunctionLoader::default().parse_all_functions(&module)?;
            Ok(module)
        }
        #[cfg(feature = "llvm")]
        SourceFormat::Cwasm => {
            let module =
                llvm_gen::aot::parse_aot_meta(buf).map_err(|e| ModuleError::Msg(e.to_string()))?;
            llvm_gen::FunctionLoader::default().parse_all_functions(&module)?;
            Ok(module)
        }
        #[cfg(not(feature = "llvm"))]
        SourceFormat::Cwasm => Err(ModuleError::Msg(
            "Cwasm files are not supported without the `llvm` feature.".to_string(),
        )),
    }
}
