use crate::{InterpreterContext, InterpreterError};
use module::{basic_block::BasicBlockID, instructions::VariableID, objects::value::ValueRaw};
use wasm_types::{TableIdx, TypeIdx};

// type_idx,
// selector_var,
// table_idx,
// return_bb,
// call_params

pub(super) fn handle_call_indirect(
    _ctx: &mut InterpreterContext,
    _type_idx: TypeIdx,
    _selector_var: VariableID,
    _table_idx: TableIdx,
    _return_bb: BasicBlockID,
    _call_params: &[VariableID],
    _return_vars: &[VariableID],
) -> Result<Option<Vec<ValueRaw>>, InterpreterError> {
    todo!("call raw pointer with parameters and expect return values");
}
