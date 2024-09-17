use module::{
    basic_block::BasicBlock,
    instructions::Variable,
    objects::function::{Function, FunctionIR},
};
use wasm_types::ValType;

pub(crate) fn get_bbs_from_function(f: &Function) -> &Vec<BasicBlock> {
    if let Some(FunctionIR { bbs, .. }) = f.get_ir() {
        bbs
    } else {
        todo!("Imported functions")
    }
}

pub(crate) fn get_locals_from_function(f: &Function) -> &Vec<ValType> {
    if let Some(FunctionIR { locals, .. }) = f.get_ir() {
        locals
    } else {
        todo!("Imported functions")
    }
}
