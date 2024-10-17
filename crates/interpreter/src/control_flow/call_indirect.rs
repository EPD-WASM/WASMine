use crate::{control_flow::call::handle_call, InterpreterContext, InterpreterError};
use log;
use module::{basic_block::BasicBlockID, instructions::VariableID, objects::value::ValueRaw};
use wasm_types::{TableIdx, TypeIdx};

pub(super) fn handle_call_indirect(
    ctx: &mut InterpreterContext,
    type_idx: TypeIdx,
    selector_var: VariableID,
    table_idx: TableIdx,
    return_bb: BasicBlockID,
    call_params: &[VariableID],
    return_vars: &[VariableID],
) -> Result<Option<Vec<ValueRaw>>, InterpreterError> {
    log::trace!("Handling call indirect");
    let selector = ctx.stack.last().unwrap().vars.get(selector_var).as_u32();

    let fn_ptr =
        unsafe { runtime_interface::indirect_call(ctx.exec_ctx, table_idx, type_idx, selector) };

    log::trace!("Indirect call fn ptr: {:?}", fn_ptr);

    // internal function pointers are just function indices
    let fn_idx =
        unsafe { std::mem::transmute::<&std::ffi::c_void, u64>(fn_ptr.as_ref()) } as u32 - 1;

    log::trace!("Indirect call to function idx: {}", fn_idx);

    handle_call(ctx, fn_idx, return_bb, call_params, return_vars)
}
