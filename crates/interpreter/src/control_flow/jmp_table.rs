use crate::{control_flow::util::break_util, InterpreterContext, InterpreterError};
use module::{instructions::VariableID, objects::value::ValueRaw};

pub(super) fn handle_jmp_table(
    ctx: &mut InterpreterContext,
    cond_var: VariableID,
    targets: &[u32],
    default_target: u32,
) -> Result<Option<Vec<ValueRaw>>, InterpreterError> {
    let stack_frame = ctx.stack.last_mut().unwrap();
    let cond: u32 = stack_frame.vars.get(cond_var).into();
    let target = if let Some(target) = targets.get(cond as usize) {
        *target
    } else {
        default_target
    };
    break_util(ctx, target);
    Ok(None)
}
