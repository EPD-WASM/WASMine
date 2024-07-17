use ir::{instructions::r#return, InstructionDecoder};

use crate::{InterpreterContext, InterpreterError, StackFrame, VariableStore};

use super::util::call_util;

pub(super) fn handle_call(
    ctx: &mut InterpreterContext,
    func_idx: u32,
    return_bb: u32,
    call_params: &[u32],
    return_vars: &[u32],
) -> Result<Option<Vec<u64>>, InterpreterError> {
    call_util(ctx, func_idx, call_params, return_bb, return_vars);

    let vals = return_vars
        .iter()
        .map(|&var| ctx.stack.last().unwrap().vars.get(var))
        .collect();

    Ok(Some(vals))
}
