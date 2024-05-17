use std::fs::File;
use std::io::{Read, Result};

/// opens a webassembly module file with the given name in the current working directory and returns a reader
///
/// @param name the name of the module to open
/// @return a biarystreamreader for the module file
#[allow(unused)]
pub fn load_module(name: &str) -> Result<impl Read> {
    let current_dir = std::env::current_dir()?;
    let filename = current_dir.join(name);
    let file = File::open(filename)?;
    Ok(file)
}
