use module::{instructions::VariableID, objects::value::ValueRaw, InstructionDecoder};

use crate::{util::get_bbs_from_function, InterpreterContext, InterpreterError};

pub(super) fn handle_return(
    ctx: &mut InterpreterContext,
    return_vars: &[VariableID],
) -> Result<Option<Vec<ValueRaw>>, InterpreterError> {
    let old_stack_frame = ctx.stack.pop().unwrap();

    let mut return_values = Vec::new();
    let mut sorted_ret_ids = return_vars.to_vec();
    sorted_ret_ids.sort();

    for var_id in sorted_ret_ids {
        return_values.push(old_stack_frame.vars.get(var_id));
    }

    // println!("return values: {:?}", return_values);

    // create a new decoder with the return_bb
    let stack_frame = match ctx.stack.last_mut() {
        Some(sf) => sf,
        None => return Ok(Some(return_values)),
    };

    debug_assert_eq!(old_stack_frame.return_vars.len(), return_values.len());

    for (&var_id, &var_val) in old_stack_frame.return_vars.iter().zip(return_values.iter()) {
        stack_frame.vars.set(var_id, var_val);
    }

    // stack_frame.vars.vars.extend(return_values.clone());

    // println!(
    //     "returning to bb: {} of function: {}",
    //     stack_frame.bb_id, stack_frame.fn_idx
    // );
    let bbs = get_bbs_from_function(&ctx.module.meta.functions[stack_frame.fn_idx as usize]);
    let new_decoder = InstructionDecoder::new(
        bbs.iter()
            .find(|bb| bb.id == stack_frame.bb_id)
            .unwrap()
            .instructions
            .clone(),
    );

    stack_frame.decoder = new_decoder;

    Ok(Some(return_values))
}
