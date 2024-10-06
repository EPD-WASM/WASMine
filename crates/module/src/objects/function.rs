use crate::ModuleMetadata;
use rkyv::{Archive, Deserialize, Serialize};
use wasm_types::{FuncIdx, TypeIdx};

#[derive(Debug, Clone, Deserialize, Serialize, Archive)]
pub struct Function {
    pub type_idx: u32,
    pub source: FunctionSource,
}

#[derive(Debug, Clone, Deserialize, Serialize, Archive)]
pub enum FunctionSource {
    Import(FunctionImport),
    Wasm(FunctionUnparsed),
}

impl Function {
    pub fn query_function_name(func_idx: FuncIdx, module: &ModuleMetadata) -> Option<&str> {
        module.exports.find_function_name(func_idx)
    }

    pub fn debug_function_name(func_idx: FuncIdx, module: &ModuleMetadata) -> String {
        Self::query_function_name(func_idx, module)
            .map(|s| s.to_string())
            .unwrap_or(format!("<anonymous:{func_idx}>"))
    }

    pub fn create_raw_wasm(type_idx: TypeIdx, offset: usize, size: usize) -> Self {
        Self {
            type_idx,
            source: FunctionSource::Wasm(FunctionUnparsed { offset, size }),
        }
    }

    pub fn create_import(type_idx: TypeIdx, import_idx: u32) -> Self {
        Self {
            type_idx,
            source: FunctionSource::Import(FunctionImport { import_idx }),
        }
    }

    pub fn placeholder(type_idx: TypeIdx) -> Self {
        Self {
            type_idx,
            source: FunctionSource::Wasm(FunctionUnparsed {
                offset: usize::MAX,
                size: usize::MAX,
            }),
        }
    }

    pub fn is_placeholder(&self) -> bool {
        matches!(
            self.source,
            FunctionSource::Wasm(FunctionUnparsed {
                offset: usize::MAX,
                size: usize::MAX
            })
        )
    }
}

// /// Intermediate representation of the function.
// #[derive(Debug, Clone, Default, Deserialize, Serialize, Archive)]
// pub struct FunctionIR {
//     pub locals: Vec<ValType>,
//     pub bbs: Vec<BasicBlock>,
//     pub num_vars: usize,
// }

/// Function import information.
#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub struct FunctionImport {
    pub import_idx: u32,
}

/// Unparsed binary wasm function code section. (offset into memory region (mmaped for file), length)
#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub struct FunctionUnparsed {
    pub offset: usize,
    pub size: usize,
}

// /// Precompiled LLVM function, stored in memory.
// ///
// /// Note: This is used for AOT compiled functions.
// ///       The stored value is a pointer + size to an LLVM memory buffer of the object file containing the function's symbol.
// #[derive(Debug, Clone, Archive, Deserialize, Serialize)]
// pub struct FunctionLLVM {
//     pub offset: u64,
//     pub size: usize,
// }
