use module::{
    instructions::{FunctionIR, VariableID},
    objects::{
        function::{FunctionImport, FunctionSource},
        value::ValueRaw,
    },
    InstructionDecoder,
};

use crate::{InterpreterContext, InterpreterError, InterpreterFunc};

pub(super) fn handle_return(
    ctx: &mut InterpreterContext,
    return_vars: &[VariableID],
) -> Result<Option<Vec<ValueRaw>>, InterpreterError> {
    let old_stack_frame = ctx.stack.pop().unwrap();
    ctx.exec_ctx.recursion_size.saturating_sub(1);

    log::trace!("old stack frame: {:#?}", &old_stack_frame);
    log::trace!("current stack: {:#?}", &ctx.stack);

    let mut return_values = Vec::new();
    let mut sorted_ret_ids = return_vars.to_vec();
    sorted_ret_ids.sort();

    for var_id in sorted_ret_ids {
        return_values.push(old_stack_frame.vars.get(var_id));
    }

    // create a new decoder with the return_bb
    let fn_idx = match ctx.stack.last_mut() {
        Some(sf) => sf,
        None => return Ok(Some(return_values)),
    }
    .fn_idx;

    let func = {
        let ir: &Vec<FunctionIR> = &ctx.ir;

        let fn_meta = match (&ctx).module.meta.functions.get(fn_idx as usize) {
            Some(meta) => meta,
            None => return Err(InterpreterError::FunctionNotFound(fn_idx)),
        };

        match &fn_meta.source {
            FunctionSource::Import(FunctionImport { import_idx }) => {
                InterpreterFunc::Import(*import_idx)
            }
            FunctionSource::Wasm(_) => InterpreterFunc::IR(&ir[fn_idx as usize]),
        }
    };

    let fn_ir = match func {
        InterpreterFunc::IR(function_ir) => function_ir,
        InterpreterFunc::Import(_) => unreachable!(),
    };

    let stack_frame = ctx.stack.last_mut().unwrap();

    debug_assert_eq!(old_stack_frame.return_vars.len(), return_values.len());

    for (&var_id, &var_val) in old_stack_frame.return_vars.iter().zip(return_values.iter()) {
        stack_frame.vars.set(var_id, var_val);
    }

    let bbs = &fn_ir.bbs;
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
