use super::basic_block::BasicBlock;
use crate::{
    instructions::Variable,
    structs::{export::Export, module::Module},
};
use std::vec::Vec;
use wasm_types::TypeIdx;

#[derive(Debug, Clone, Default)]
pub struct Function {
    pub type_idx: TypeIdx,
    pub locals: Vec<Variable>,
    pub basic_blocks: Vec<BasicBlock>,
    pub import: bool,
    pub num_vars: u32,
}

impl Function {
    pub fn query_function_name(func_idx: usize, module: &Module) -> Option<String> {
        module.exports.iter().find_map(|export| match &export {
            Export {
                name,
                desc: crate::structs::export::ExportDesc::Func(idx),
            } if *idx == func_idx as u32 => Some(name.clone()),
            _ => None,
        })
    }

    pub fn debug_function_name(func_idx: usize, module: &Module) -> String {
        Self::query_function_name(func_idx, module).unwrap_or(format!("<anonymous:{}>", func_idx))
    }

    pub fn create_empty(type_idx: TypeIdx) -> Self {
        Self {
            type_idx,
            locals: Vec::new(),
            basic_blocks: Vec::new(),
            import: false,
            num_vars: 0,
        }
    }
}
