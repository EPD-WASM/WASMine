use ir::structs::module::Module as WasmModule;
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::rc::Rc;

#[derive(Debug, thiserror::Error)]
pub enum LoaderError {
    #[error("Invalid input format: could not load file with ending \'{0}\'")]
    InvalidFileEnding(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("CBOR error: {0}")]
    BitcodeError(#[from] bitcode::Error),
}

#[derive(Clone)]
pub struct WasmLoader {
    source: Source,
}

#[derive(Clone)]
enum Source {
    File { path: PathBuf },
    Mem { buf: Vec<u8> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceFormat {
    Wasm,
    Cwasm,
}

impl SourceFormat {
    pub fn from_path(path: &Path) -> Result<Self, LoaderError> {
        let ext: &std::ffi::OsStr = path
            .extension()
            .ok_or_else(|| LoaderError::InvalidFileEnding("no extension found".to_string()))?;
        match ext.to_str().unwrap() {
            "wasm" => Ok(Self::Wasm),
            "cwasm" => Ok(Self::Cwasm),
            _ => Err(LoaderError::InvalidFileEnding(
                ext.to_str().unwrap().to_string(),
            )),
        }
    }
}

impl WasmLoader {
    pub fn from_file(path: &Path) -> Result<Self, LoaderError> {
        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            let current_dir = std::env::current_dir().unwrap();
            current_dir.join(path)
        };
        debug_assert_eq!(SourceFormat::from_path(&path)?, SourceFormat::Wasm);
        Ok(Self {
            source: Source::File { path },
        })
    }

    pub fn from_buf(buf: Vec<u8>) -> Self {
        Self {
            source: Source::Mem { buf },
        }
    }

    pub fn load<'a>(&'a self) -> Result<Box<dyn Read + 'a>, LoaderError> {
        match &self.source {
            Source::File { path } => Ok(File::open(path).map(|f| Box::new(f) as Box<dyn Read>)?),
            Source::Mem { buf } => Ok(Box::new(buf.as_slice())),
        }
    }
}

#[derive(Debug)]
pub struct CwasmLoader {
    wasm_module: Rc<WasmModule>,
    llvm_memory_buffer_offset: usize,

    // actual data, stored in memory via mmap
    file_mmap: memmap2::Mmap,
    _file: File,
    file_len: usize,
}

impl CwasmLoader {
    pub fn from_file(path: &Path) -> Result<Self, LoaderError> {
        debug_assert_eq!(SourceFormat::from_path(path)?, SourceFormat::Cwasm);
        let path = if path.is_absolute() {
            path.to_path_buf()
        } else {
            let current_dir = std::env::current_dir().unwrap();
            current_dir.join(path)
        };

        let file = std::fs::File::open(path)?;
        let file_len = file.metadata()?.len() as usize;
        let file_mmap = unsafe { memmap2::Mmap::map(&file)? };

        let wasm_module_buffer_size =
            u32::from_be_bytes(file_mmap[0..4].try_into().unwrap()) as usize;
        let llvm_memory_buffer_offset =
            u32::from_be_bytes(file_mmap[4..8].try_into().unwrap()) as usize;

        let wasm_module = Rc::new(bitcode::decode(&file_mmap[8..8 + wasm_module_buffer_size])?);
        Ok(Self {
            wasm_module,
            llvm_memory_buffer_offset,
            file_mmap,
            _file: file,
            file_len,
        })
    }

    pub fn write(
        path: &Path,
        wasm_module: Rc<WasmModule>,
        llvm_memory_buffer: &[u8],
    ) -> Result<(), LoaderError> {
        let mut file: File = std::fs::File::create(path)?;
        let wasm_module_buffer = bitcode::encode(wasm_module.as_ref());
        let llvm_obj_offset: u32 =
            (2 * std::mem::size_of::<u32>() + wasm_module_buffer.len()).next_multiple_of(2) as u32;

        file.write_all(&u32::to_be_bytes(wasm_module_buffer.len() as u32))?;
        file.write_all(&u32::to_be_bytes(llvm_obj_offset))?;

        file.write_all(&wasm_module_buffer)?;
        file.seek(std::io::SeekFrom::Start(llvm_obj_offset as u64))?;
        file.write_all(llvm_memory_buffer)?;
        Ok(())
    }

    pub fn llvm_memory_buffer(&self) -> &[u8] {
        &self.file_mmap[self.llvm_memory_buffer_offset..self.file_len]
    }

    pub fn wasm_module(&self) -> Rc<WasmModule> {
        self.wasm_module.clone()
    }
}
