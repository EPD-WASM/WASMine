#![no_main]

use libfuzzer_sys::fuzz_target;
use wasm_rt::parser::parser::Parser;
use wasm_smith::Module;

fuzz_target!(|module: Module| {
    let wasm_bytes = module.to_bytes();
    let parser = Parser::default();
    parser.parse(wasm_bytes.as_slice()).unwrap();
});
