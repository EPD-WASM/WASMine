use ir::basic_block::BasicBlockGlue;

use crate::{control_flow::util::break_util, InterpreterContext, InterpreterError};

pub(super) fn handle_jmp(
    ctx: &mut InterpreterContext,
    target: u32,
) -> Result<Option<Vec<u64>>, InterpreterError> {
    break_util(ctx, target);

    Ok(None)
}
