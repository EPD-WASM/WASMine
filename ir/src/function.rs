use super::basic_block::BasicBlock;
use crate::structs::module::Module;
use serde::{Deserialize, Serialize};
use std::vec::Vec;
use wasm_types::{FuncIdx, TypeIdx, ValType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInternal {
    pub locals: Vec<ValType>,
    pub bbs: Vec<BasicBlock>,
    pub num_vars: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionImport {
    pub import_idx: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FunctionSource {
    Internal(FunctionInternal),
    Import(FunctionImport),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    pub type_idx: u32,
    pub src: FunctionSource,
}

impl Function {
    pub fn query_function_name(func_idx: FuncIdx, module: &Module) -> Option<&str> {
        module.exports.find_function_name(func_idx)
    }

    pub fn debug_function_name(func_idx: FuncIdx, module: &Module) -> String {
        Self::query_function_name(func_idx, module)
            .map(|s| s.to_string())
            .unwrap_or(format!("<anonymous:{}>", func_idx))
    }

    pub fn create_empty(type_idx: TypeIdx) -> Self {
        Self {
            type_idx,
            src: FunctionSource::Internal(FunctionInternal {
                locals: Vec::new(),
                bbs: Vec::new(),
                num_vars: 0,
            }),
        }
    }
}
