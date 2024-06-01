/// Helper functions for complex WASM instructions
mod sync;

#[no_mangle]
pub extern "C" fn trap() -> ! {
    unimplemented!()
}
