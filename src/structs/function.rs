use crate::{instructions::Variable, wasm_types::wasm_type::TypeIdx};

use super::basic_block::BasicBlock;
use std::vec::Vec;

#[derive(Debug, Clone, Default)]
pub(crate) struct Function {
    pub(crate) type_idx: TypeIdx,
    pub(crate) locals: Vec<Variable>,
    pub(crate) basic_blocks: Vec<BasicBlock>,
    pub(crate) import: bool,
}
