use super::basic_block::BasicBlock;

#[derive(Debug, Clone, Default)]
pub(crate) struct Expression {
    pub(crate) instrs: Vec<BasicBlock>,
}
