use std::fs::File;
use std::io::{Read, Result};
use std::path::{Path, PathBuf};

pub enum Loader {
    PathLoader { path: PathBuf },
    BufLoader { buf: Vec<u8> },
}

impl Loader {
    pub fn from_file(path: &Path) -> Self {
        if path.is_absolute() {
            Loader::PathLoader {
                path: path.to_path_buf(),
            }
        } else {
            let current_dir = std::env::current_dir().unwrap();
            let filename = current_dir.join(path);
            Loader::PathLoader { path: filename }
        }
    }

    pub fn from_buf(buf: Vec<u8>) -> Self {
        Loader::BufLoader { buf }
    }

    pub fn load<'a>(&'a self) -> Result<Box<dyn Read + 'a>> {
        match self {
            Loader::PathLoader { path } => File::open(path).map(|f| Box::new(f) as Box<dyn Read>),
            Loader::BufLoader { buf } => Ok(Box::new(buf.as_slice()) as Box<dyn Read>),
        }
    }
}
