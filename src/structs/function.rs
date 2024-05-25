use super::basic_block::BasicBlock;
use crate::instructions::Variable;
use std::vec::Vec;
use wasm_types::TypeIdx;

#[derive(Debug, Clone, Default)]
pub(crate) struct Function {
    pub(crate) type_idx: TypeIdx,
    pub(crate) locals: Vec<Variable>,
    pub(crate) basic_blocks: Vec<BasicBlock>,
    pub(crate) import: bool,
}
