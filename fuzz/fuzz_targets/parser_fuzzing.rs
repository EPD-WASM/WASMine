#![no_main]

use libfuzzer_sys::fuzz_target;
use loader::WasmLoader;
use parser::parser::Parser;
use wasm_smith::Module;

fuzz_target!(|module: Module| {
    let wasm_bytes = module.to_bytes();
    let parser = Parser::default();
    let loader = WasmLoader::from_buf(wasm_bytes);
    if parser.parse(loader).is_err() {
        std::fs::write("fuzz.wasm", module.to_bytes()).unwrap();
        panic!("Failed to parse the generated module")
    }
});
