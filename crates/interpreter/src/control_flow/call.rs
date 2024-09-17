use crate::{InterpreterContext, InterpreterError};
use module::{basic_block::BasicBlockID, instructions::VariableID, objects::value::ValueRaw};
use wasm_types::FuncIdx;

use super::util::call_util;

pub(super) fn handle_call(
    ctx: &mut InterpreterContext,
    func_idx: FuncIdx,
    return_bb: BasicBlockID,
    call_params: &[VariableID],
    return_vars: &[VariableID],
) -> Result<Option<Vec<ValueRaw>>, InterpreterError> {
    call_util(ctx, func_idx, call_params, return_bb, return_vars);
    let vals = return_vars
        .iter()
        .map(|&var| ctx.stack.last().unwrap().vars.get(var))
        .collect();
    Ok(Some(vals))
}
