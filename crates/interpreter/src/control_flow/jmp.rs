use crate::{control_flow::util::break_util, InterpreterContext, InterpreterError};
use ir::{basic_block::BasicBlockID, structs::value::ValueRaw};

pub(super) fn handle_jmp(
    ctx: &mut InterpreterContext,
    target: BasicBlockID,
) -> Result<Option<Vec<ValueRaw>>, InterpreterError> {
    break_util(ctx, target);
    Ok(None)
}
