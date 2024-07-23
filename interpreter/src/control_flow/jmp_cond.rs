use crate::{control_flow::util::break_util, InterpreterContext, InterpreterError};
use ir::structs::value::ValueRaw;

pub(super) fn handle_jmp_cond(
    ctx: &mut InterpreterContext,
    cond_var: u32,
    target_if_true: u32,
    target_if_false: u32,
) -> Result<Option<Vec<ValueRaw>>, InterpreterError> {
    let stack_frame = ctx.stack.last_mut().unwrap();
    let cond: u32 = stack_frame.vars.get(cond_var).into();
    let target = if cond != 0 {
        target_if_true
    } else {
        target_if_false
    };
    break_util(ctx, target);
    Ok(None)
}
