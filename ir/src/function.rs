use super::basic_block::BasicBlock;
use crate::{
    instructions::Variable,
    structs::{export::Export, module::Module},
};
use std::vec::Vec;
use wasm_types::{FuncIdx, TypeIdx};

#[derive(Debug, Clone)]
pub struct FunctionInternal {
    pub locals: Vec<Variable>,
    pub bbs: Vec<BasicBlock>,
    pub num_vars: u32,
}

#[derive(Debug, Clone)]
pub struct FunctionImport {
    pub import_idx: u32,
}

#[derive(Debug, Clone)]
pub enum FunctionSource {
    Internal(FunctionInternal),
    Import(FunctionImport),
}

#[derive(Debug, Clone)]
pub struct Function {
    pub type_idx: u32,
    pub src: FunctionSource,
}

impl Function {
    pub fn query_function_name(func_idx: FuncIdx, module: &Module) -> Option<String> {
        module.exports.iter().find_map(|export| match &export {
            Export {
                name,
                desc: crate::structs::export::ExportDesc::Func(idx),
            } if *idx == func_idx => Some(name.clone()),
            _ => None,
        })
    }

    pub fn debug_function_name(func_idx: FuncIdx, module: &Module) -> String {
        Self::query_function_name(func_idx, module).unwrap_or(format!("<anonymous:{}>", func_idx))
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
