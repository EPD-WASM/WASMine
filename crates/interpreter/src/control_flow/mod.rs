use call::handle_call;
use call_indirect::handle_call_indirect;
use jmp::handle_jmp;
use jmp_cond::handle_jmp_cond;
use jmp_table::handle_jmp_table;
use module::{
    basic_block::BasicBlockGlue,
    instructions::FunctionIR,
    objects::{
        function::{FunctionImport, FunctionSource},
        value::ValueRaw,
    },
};
use r#return::handle_return;

use crate::{InterpreterContext, InterpreterError, InterpreterFunc};

mod call;
mod call_indirect;
mod jmp;
mod jmp_cond;
mod jmp_table;
mod r#return;
pub(super) mod util;

pub(super) trait GlueHandler {
    fn handle(
        &self,
        ctx: &mut InterpreterContext,
    ) -> Result<Option<Vec<ValueRaw>>, InterpreterError>;
}

impl GlueHandler for BasicBlockGlue {
    fn handle(
        &self,
        ctx: &mut InterpreterContext,
    ) -> Result<Option<Vec<ValueRaw>>, InterpreterError> {
        log::trace!("Handling basic block glue: {:?}", self);
        let res = match self {
            BasicBlockGlue::Jmp { target, .. } => handle_jmp(ctx, *target),
            BasicBlockGlue::JmpCond {
                cond_var,
                target_if_true,
                target_if_false,
                ..
            } => handle_jmp_cond(ctx, *cond_var, *target_if_true, *target_if_false),
            BasicBlockGlue::JmpTable {
                selector_var,
                targets,
                default_target,
                ..
            } => handle_jmp_table(ctx, *selector_var, targets, *default_target),
            BasicBlockGlue::Call {
                func_idx,
                return_bb,
                call_params,
                return_vars,
            } => handle_call(ctx, *func_idx, *return_bb, call_params, return_vars),
            BasicBlockGlue::CallIndirect {
                type_idx,
                selector_var,
                table_idx,
                return_bb,
                call_params,
                return_vars,
            } => handle_call_indirect(
                ctx,
                *type_idx,
                *selector_var,
                *table_idx,
                *return_bb,
                call_params,
                return_vars,
            ),
            BasicBlockGlue::Return { return_vars } => return handle_return(ctx, return_vars),
            BasicBlockGlue::ElseMarker { .. } => Ok(None), // no-op
            BasicBlockGlue::Unreachable => Err(InterpreterError::Unreachable),
        };

        log::debug!("Terminator res: {:?}", &res);

        let fn_idx = if let Some(stack_frame) = ctx.stack.last_mut() {
            stack_frame.fn_idx
        } else {
            return res;
        };

        // Here we have just started a new basic block. Resolve PhiNodes in its inputs.

        let func: Result<_, InterpreterError> = {
            let ir: &Vec<FunctionIR> = &ctx.ir;

            let fn_meta = match (&ctx).module.meta.functions.get(fn_idx as usize) {
                Some(meta) => meta,
                None => return Err(InterpreterError::FunctionNotFound(fn_idx)),
            };

            match &fn_meta.source {
                FunctionSource::Import(FunctionImport { import_idx }) => {
                    Ok(InterpreterFunc::Import(*import_idx))
                }
                FunctionSource::Wasm(_) => Ok(InterpreterFunc::IR(&ir[fn_idx as usize])),
            }
        };
        let fn_ir = match func.unwrap() {
            InterpreterFunc::IR(function_ir) => function_ir,
            InterpreterFunc::Import(_) => unreachable!(),
        };

        let stack_frame = ctx.stack.last_mut().unwrap();

        let bbs = &fn_ir.bbs;
        let bb = bbs
            .iter()
            .find(|bb| bb.id == stack_frame.bb_id)
            .unwrap_or_else(|| panic!("Basic block with ID {} not found", stack_frame.bb_id));

        log::trace!("Resolving PhiNodes in inputs: {:?}", bb.inputs);
        for phi_node in &bb.inputs {
            log::trace!("Resolving PhiNode: {:?}", phi_node);
            let (_, var_idx) = *phi_node
                .inputs
                .iter()
                .find(|(bb, _)| stack_frame.last_bb_id == *bb)
                .unwrap_or_else(|| {
                    panic!(
                        "PhiNode {:?} has no input for bb {}",
                        phi_node, stack_frame.last_bb_id
                    )
                });

            let value = stack_frame.vars.get(var_idx);
            stack_frame.vars.set(phi_node.out, value);
        }

        res
    }
}
