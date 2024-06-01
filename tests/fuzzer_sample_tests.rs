// test all samples previously found to be erroneous by the fuzzer

use std::path::PathBuf;
use wast::Wat;

#[test]
fn test_fuzzer_samples() {
    std::fs::read_dir(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fuzzer_samples"),
    )
    .unwrap()
    .map(|entry| entry.unwrap().path())
    .filter(|path| path.extension().is_some() && path.extension().unwrap() == "wat")
    .for_each(|file_path| {
        let wasm_text = std::fs::read_to_string(&file_path).unwrap();
        let wast_parsebuf = wast::parser::ParseBuffer::new(&wasm_text).unwrap();
        let mut wast_repr: Wat = wast::parser::parse(&wast_parsebuf).unwrap();
        let wasm_bytes = wast_repr.encode().unwrap();
        let parser = parser::parser::Parser::default();
        let loader = loader::Loader::from_buf(wasm_bytes.clone());
        if let Err(e) = parser.parse(loader) {
            panic!("Failed to parse wasm file {:?}: {}", file_path, e)
        }
    });
}
