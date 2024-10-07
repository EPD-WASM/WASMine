use module::Module as WasmModule;
use resource_buffer::ResourceBuffer;
use rkyv::ser::{
    serializers::{
        AllocScratch, CompositeSerializer, FallbackScratch, HeapScratch, SharedSerializeMap,
        WriteSerializer,
    },
    Serializer,
};
use std::{
    fs::File,
    io::{Seek, SeekFrom, Write},
    path::Path,
    sync::RwLock,
};

#[derive(Debug, thiserror::Error)]
pub enum AOTError {
    #[error("ResourceBuffer error: {0}")]
    ResourceBufferError(#[from] resource_buffer::ResourceBufferError),

    #[error("AOT error: {0}")]
    Msg(String),

    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
}

/// Precompiled LLVM function, stored in memory.
///
/// Note: This is used for AOT compiled functions.
///       The stored value is a pointer + size to an LLVM memory buffer of the object file containing the function's symbol.
#[derive(Debug, Clone)]
pub struct AOTFunctions {
    pub offset: u64,
    pub size: usize,
}

pub fn parse_aot_module(buffer: ResourceBuffer) -> Result<WasmModule, AOTError> {
    let wasm_module = parse_aot_meta(buffer)?;
    parse_aot_functions(&wasm_module)?;
    Ok(wasm_module)
}

pub fn parse_aot_meta(buffer: ResourceBuffer) -> Result<WasmModule, AOTError> {
    log::debug!("Loading aot module meta using `llvm-gen`.");
    let input = buffer.get()?;
    let wasm_module_buffer_size = u32::from_be_bytes(input[0..4].try_into().unwrap()) as usize;

    #[allow(never_type_fallback_flowing_into_unsafe)]
    let module_meta = unsafe { rkyv::from_bytes_unchecked(&input[8..8 + wasm_module_buffer_size]) }
        .map_err(|e| AOTError::Msg(format!("Failed to decode module metadata: {e}")))?;
    Ok(WasmModule {
        meta: module_meta,
        source: buffer,
        artifact_registry: Default::default(),
    })
}

pub fn parse_aot_functions(wasm_module: &WasmModule) -> Result<(), AOTError> {
    log::debug!("Loading aot llvm functions using `llvm-gen`.");
    let mut artifacts_ref = wasm_module.artifact_registry.write().unwrap();
    if artifacts_ref.contains_key("llvm-obj") {
        log::info!("LLVM aot functions already parsed by `llvm-gen`. Skipping.");
        return Ok(());
    }

    let input = wasm_module.source.get()?;
    let llvm_memory_buffer_offset = u32::from_be_bytes(input[4..8].try_into().unwrap()) as usize;
    let llvm_memory_buffer = &input[llvm_memory_buffer_offset..];
    artifacts_ref.insert(
        "llvm-obj".to_string(),
        RwLock::new(Box::new(AOTFunctions {
            offset: llvm_memory_buffer.as_ptr() as u64,
            size: input.len() - llvm_memory_buffer_offset,
        })),
    );
    Ok(())
}

pub fn store_aot_module(
    module: &WasmModule,
    llvm_memory_buffer: impl AsRef<[u8]>,
    output_path: impl AsRef<Path>,
) -> Result<(), AOTError> {
    let mut out_file = File::create(output_path)?;
    out_file.seek(SeekFrom::Start(8))?;

    let mut serializer = CompositeSerializer::new(
        WriteSerializer::new(&mut out_file),
        <FallbackScratch<HeapScratch<1024>, AllocScratch>>::default(),
        SharedSerializeMap::default(),
    );
    debug_assert_eq!(serializer.pos(), 0);
    serializer.serialize_value(&module.meta).unwrap();
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
