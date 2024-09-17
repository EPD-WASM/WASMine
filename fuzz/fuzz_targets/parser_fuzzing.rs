#![no_main]

use libfuzzer_sys::fuzz_target;
use runtime_lib::{FunctionLoader, ModuleMetaLoader, ResourceBuffer, WasmModule};
use wasm_smith::Module;

fuzz_target!(|module: Module| {
    let wasm_bytes = module.to_bytes();
    let source = ResourceBuffer::from_wasm_buf(wasm_bytes.clone());
    let mut module = WasmModule::new(source);
    if module.load_meta(ModuleMetaLoader).is_err()
        || module.load_all_functions(FunctionLoader).is_err()
    {
        std::fs::write("fuzz.wasm", wasm_bytes).unwrap();
        panic!("Failed to parse the generated module")
    }
});
