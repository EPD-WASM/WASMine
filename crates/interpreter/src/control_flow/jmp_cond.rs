use crate::{control_flow::util::break_util, InterpreterContext, InterpreterError};
use module::{instructions::VariableID, objects::value::ValueRaw, BasicBlock, BasicBlockID};

pub(super) fn handle_jmp_cond(
    ctx: &mut InterpreterContext,
    cond_var: VariableID,
    target_if_true: BasicBlockID,
    target_if_false: BasicBlockID,
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
