// use crate::{
//     func::Function,
//     linker::{DependencyName, FunctionDependency},
// };
// use runtime_interface::RawFunctionPtr;
// use std::collections::HashMap;
// use wasm_types::{FuncType, NumType, ValType};

// /// Reference: https://github.com/WebAssembly/WASI/blob/89646e96b8f61fc57ae4c7d510d2dce68620e6a4/legacy/preview1/docs.md
// mod functions;
// mod types;

// static I32: ValType = ValType::Number(NumType::I32);

// // #[allow(clippy::fn_to_numeric_cast)]
// // pub(crate) fn collect_available_imports() -> Vec<FunctionDependency> {
// //     // #[rustfmt::skip]
// //     // return vec![
// //     //     FunctionDependency {
// //     //         name: DependencyName {
// //     //             module: "wasi_snapshot_preview1".to_string(),
// //     //             name: "fd_filestat_get".to_string(),
// //     //         },
// //     //         func: Function::from
// //     //         }
// //     //     ];
// //     // into(), /* inputs: fd + return_ptr, outputs: errno */ function_type: FuncType(vec![I32, I32], vec![I32]), callable: RawFunctionPtr::new(functions::fd_filestat_get as _).unwrap(), execution_context: None},
// //     //     )];
// // }
