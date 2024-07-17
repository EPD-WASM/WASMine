use std::os::linux::raw::stat;

use ir::{utils::numeric_transmutes::Bit64, InstructionDecoder};

use crate::{
    control_flow::util::{break_util, call_util},
    InterpreterContext, InterpreterError, StackFrame, VariableStore,
};

// type_idx,
// selector_var,
// table_idx,
// return_bb,
// call_params

pub(super) fn handle_call_indirect(
    ctx: &mut InterpreterContext,
    type_idx: u32,
    selector_var: u32,
    table_idx: u32,
    return_bb: u32,
    call_params: &[u32],
    return_vars: &[u32],
) -> Result<Option<Vec<u64>>, InterpreterError> {
    let selector = ctx.stack.last().unwrap().vars.get(selector_var).trans_u32();

    let fn_ptr =
        unsafe { runtime_interface::indirect_call(ctx.exec_ctx, table_idx, type_idx, selector) };

    let fn_types = &ctx.module.function_types[type_idx as usize];
    todo!("call raw pointer with parameters and expect return values");
}
