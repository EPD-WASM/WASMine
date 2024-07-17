use ir::InstructionDecoder;
use wasm_types::LabelIdx;

use crate::{util, InterpreterContext, StackFrame, VariableStore};

pub(super) fn break_util(ctx: &mut InterpreterContext, label_idx: LabelIdx) {
    let stack_frame = ctx.stack.last_mut().unwrap();
    let last_bb_idx = stack_frame.bb_id;

    stack_frame.bb_id = label_idx;
    stack_frame.last_bb_id = last_bb_idx;

    let function_idx = stack_frame.fn_idx;
    let current_fn = &ctx.module.ir.functions[function_idx as usize];

    // TODO: store pointer to entry block. This is currently always BB0, but this might change in the future

    let basic_block = util::get_bbs_from_function(&current_fn)
        .iter()
        .find(|bb| bb.id == label_idx)
        .unwrap();

    let instrs = basic_block.instructions.clone();

    stack_frame.decoder = InstructionDecoder::new(instrs);
}

pub(super) fn call_util(
    ctx: &mut InterpreterContext,
    func_idx: u32,
    call_params: &[u32],
    return_bb: u32,
    return_vars: &[u32],
) {
    let stack_frame = ctx.stack.last_mut().unwrap();
    let func = &ctx.module.ir.functions[func_idx as usize];

    let bbs = util::get_bbs_from_function(func);
    let mut new_stack_frame = StackFrame {
        fn_idx: func_idx,
        fn_local_vars: VariableStore::new(Vec::new()),
        bb_id: bbs[0].id,
        last_bb_id: 0,
        return_vars: Vec::from(return_vars),
        decoder: InstructionDecoder::new(bbs[0].instructions.clone()),
        vars: VariableStore::new(Vec::new()),
    };

    // println!("calling function with idx: {} with parameters:", func_idx);
    for (idx, &param) in call_params.iter().enumerate() {
        let var = stack_frame.vars.get(param);
        // println!("{}: {}", idx, var);
        new_stack_frame.fn_local_vars.set(idx as u32, var);
    }

    stack_frame.last_bb_id = stack_frame.bb_id;
    stack_frame.bb_id = return_bb;

    ctx.stack.push(new_stack_frame);
}
