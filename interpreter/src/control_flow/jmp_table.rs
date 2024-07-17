use crate::{control_flow::util::break_util, InterpreterContext, InterpreterError};

pub(super) fn handle_jmp_table(
    ctx: &mut InterpreterContext,
    cond_var: u32,
    targets: &[u32],
    default_target: u32,
) -> Result<Option<Vec<u64>>, InterpreterError> {
    let stack_frame = ctx.stack.last_mut().unwrap();
    let cond = stack_frame.vars.get(cond_var);

    let target = if let Some(target) = targets.get(cond as usize) {
        *target
    } else {
        default_target
    };

    break_util(ctx, target);

    Ok(None)
}