use std::fs::File;
use std::path::Path;
use std::pin::Pin;

#[derive(Debug, thiserror::Error)]
pub enum ResourceBufferError {
    #[error("Invalid input format: could not load file with ending \'{0}\'")]
    InvalidFileEnding(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub struct ResourceBuffer {
    source: Source,
    kind: SourceFormat,
}

enum Source {
    File {
        file_mmap: memmap2::Mmap,
        // keep file open to keep mmap alive
        _file: File,
        file_len: usize,
    },
    Mem {
        buf: Pin<Box<Vec<u8>>>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceFormat {
    Wasm,
    Cwasm,
}

impl SourceFormat {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, ResourceBufferError> {
        let ext: &std::ffi::OsStr = path.as_ref().extension().ok_or_else(|| {
            ResourceBufferError::InvalidFileEnding("no extension found".to_string())
        })?;
        match ext.to_str().unwrap() {
            "wasm" => Ok(Self::Wasm),
            "cwasm" => Ok(Self::Cwasm),
            _ => Err(ResourceBufferError::InvalidFileEnding(
                ext.to_str().unwrap().to_string(),
            )),
        }
    }
}

impl ResourceBuffer {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, ResourceBufferError> {
        let path = if path.as_ref().is_absolute() {
            path.as_ref().to_path_buf()
        } else {
            let current_dir = std::env::current_dir().unwrap();
            current_dir.join(path)
        };
        let source_format = SourceFormat::from_path(&path)?;

        let file = std::fs::File::open(path)?;
        let file_len = file.metadata()?.len() as usize;
        let file_mmap = unsafe { memmap2::Mmap::map(&file)? };
        Ok(Self {
            source: Source::File {
                _file: file,
                file_len,
                file_mmap,
            },
            kind: source_format,
        })
    }

    pub fn from_wasm_buf(buf: Vec<u8>) -> Self {
        Self {
            source: Source::Mem { buf: Box::pin(buf) },
            kind: SourceFormat::Wasm,
        }
    }

    pub fn get<'a>(&'a self) -> &'a [u8] {
        match &self.source {
            Source::File {
                file_len,
                file_mmap,
                ..
            } => &file_mmap[..*file_len],
            Source::Mem { buf } => &buf,
        }
    }

    pub fn kind(&self) -> SourceFormat {
        self.kind
    }
}
