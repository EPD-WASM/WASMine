#![no_main]

use libfuzzer_sys::fuzz_target;
use runtime_lib::{FunctionLoader, FunctionLoaderInterface, Parser};
use wasm_smith::Module;

fuzz_target!(|module: Module| {
    let wasm_bytes = module.to_bytes();
    let module = Parser::parse_from_buf(wasm_bytes.clone());
    if let Err(e) = module {
        panic!("Failed to parse wasm file meta: {e}")
    }
    let module = module.unwrap();

    if let Err(e) = FunctionLoader.parse_all_functions(&module) {
        panic!("Failed to parse IR functions: {e}")
    }

    if let Err(e) = llvm_gen::Translator::translate_module_meta(&module) {
        panic!("Failed to translate module meta in: {e}")
    }

    if let Err(e) = llvm_gen::FunctionLoader.parse_all_functions(&module) {
        panic!("Failed to parse llvm functions from: {e}")
    }
});
