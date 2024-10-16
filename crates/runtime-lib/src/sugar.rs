use module::{Module, ModuleError};
use resource_buffer::{ResourceBuffer, SourceFormat};
use std::path::Path;

pub fn module_from_file(file: &Path) -> Result<Module, ModuleError> {
    let buf = ResourceBuffer::from_file(file)?;
    match buf.kind() {
        SourceFormat::Wasm => {
            parser::Parser::parse(buf).map_err(|e| ModuleError::Msg(e.to_string()))
        }
        #[cfg(feature = "llvm")]
        SourceFormat::Cwasm => {
            llvm_gen::aot::parse_aot_meta(buf).map_err(|e| ModuleError::Msg(e.to_string()))
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
            parser::Parser::parse(buf).map_err(|e| ModuleError::Msg(e.to_string()))
        }
        #[cfg(feature = "llvm")]
        SourceFormat::Cwasm => {
            llvm_gen::aot::parse_aot_meta(buf).map_err(|e| ModuleError::Msg(e.to_string()))
        }
        #[cfg(not(feature = "llvm"))]
        SourceFormat::Cwasm => Err(ModuleError::Msg(
            "Cwasm files are not supported without the `llvm` feature.".to_string(),
        )),
    }
}
