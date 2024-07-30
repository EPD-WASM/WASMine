use std::{
    borrow::Cow,
    ffi::{CStr, CString},
};

use ir::structs::module::Module;
use wasm_types::FuncIdx;

// piratet from the inkwell library
pub(crate) fn to_c_str(mut s: &str) -> Cow<'_, CStr> {
    if s.is_empty() {
        s = "\0";
    }

    // Start from the end of the string as it's the most likely place to find a null byte
    if !s.chars().rev().any(|ch| ch == '\0') {
        return Cow::from(CString::new(s).expect("unreachable since null bytes are checked"));
    }

    unsafe { Cow::from(CStr::from_ptr(s.as_ptr() as *const _)) }
}

pub(crate) fn build_llvm_function_name(
    function_idx: FuncIdx,
    wasm_module: &Module,
    is_export: bool,
) -> String {
    if is_export {
        // make actual function name available to the public
        ir::function::Function::query_function_name(function_idx, wasm_module)
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("func_{}", function_idx))
    } else {
        // internally, we simply use the func_idx as the function name (unique-per-module)
        format!("{}", function_idx)
    }
}
