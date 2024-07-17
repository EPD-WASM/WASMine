use ir::{
    basic_block::BasicBlock,
    function::{Function, FunctionSource},
    instructions::Variable,
};
use wasm_types::ValType;

pub(crate) fn get_bbs_from_function(f: &Function) -> &Vec<BasicBlock> {
    match &f.src {
        FunctionSource::Internal(f) => &f.bbs,
        FunctionSource::Import(_) => todo!("Imported functions"),
    }
}

pub(crate) fn get_locals_from_function(f: &Function) -> &Vec<ValType> {
    match &f.src {
        FunctionSource::Internal(f) => &f.locals,
        FunctionSource::Import(_) => todo!("Imported functions"),
    }
}
