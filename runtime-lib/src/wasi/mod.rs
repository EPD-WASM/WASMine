use runtime_interface::RTImport;
use std::collections::HashMap;
use wasm_types::{NumType, ValType};

/// Reference: https://github.com/WebAssembly/WASI/blob/89646e96b8f61fc57ae4c7d510d2dce68620e6a4/legacy/preview1/docs.md
mod functions;
mod types;

static I32: ValType = ValType::Number(NumType::I32);

pub(crate) fn collect_available_imports() -> HashMap<&'static str, RTImport> {
    #[rustfmt::skip]
    return HashMap::from([
        ("wasi_snapshot_preview1.fd_filestat_get", RTImport {name: "fd_filestat_get", /* inputs: fd + return_ptr, outputs: errno */ function_type: (vec![I32, I32], vec![I32]), callable: functions::fd_filestat_get as *const u8},
    )]);
}
