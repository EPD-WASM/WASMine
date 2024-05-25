use super::basic_block::BasicBlock;
use crate::{
    instructions::Variable,
    structs::{export::Export, module::Module},
};
use std::vec::Vec;
use wasm_types::TypeIdx;

#[derive(Debug, Clone, Default)]
pub(crate) struct Function {
    pub(crate) type_idx: TypeIdx,
    pub(crate) locals: Vec<Variable>,
    pub(crate) basic_blocks: Vec<BasicBlock>,
    pub(crate) import: bool,
}

impl Function {
    pub(crate) fn query_function_name(func_idx: usize, module: &Module) -> String {
        module
            .exports
            .iter()
            .find_map(|export| match &export {
                Export {
                    name,
                    desc: crate::structs::export::ExportDesc::Func(idx),
                } if *idx == func_idx as u32 => Some(name.clone()),
                _ => None,
            })
            .unwrap_or(format!("<anonymous:{}>", func_idx))
    }
}
