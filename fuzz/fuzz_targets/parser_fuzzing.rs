#![no_main]

use libfuzzer_sys::fuzz_target;
use parser::parser::Parser;
use wasm_smith::Module;

fuzz_target!(|module: Module| {
    let wasm_bytes = module.to_bytes();
    let parser = Parser::default();
    if parser.parse(wasm_bytes.as_slice()).is_err() {
        std::fs::write("fuzz.wasm", wasm_bytes.as_slice()).unwrap();
        panic!("Failed to parse the generated module")
    }
});
