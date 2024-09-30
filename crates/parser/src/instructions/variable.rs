use super::*;
use crate::parsable::Parse;
use wasm_types::*;

pub(crate) fn local_get(
    ctxt: &mut Context,
    i: &mut WasmBinaryReader,
    o: &mut dyn InstructionConsumer,
) -> ParseResult {
    let local_idx = LocalIdx::parse(i)?;
    let local_ty = match ctxt.locals.get(local_idx as usize) {
        Some(local_ty) => local_ty,
        None => {
            return Err(ValidationError::Msg(format!(
                "local variable with index {local_idx} not found",
            ))
            .into());
        }
    };

    let out = ctxt.create_var(*local_ty);
    o.write_local_get(LocalGetInstruction {
        out1: out.id,
        local_idx,
    });
    ctxt.push_var(out);
    Ok(())
}

pub(crate) fn global_get(
    ctxt: &mut Context,
    i: &mut WasmBinaryReader,
    o: &mut dyn InstructionConsumer,
) -> ParseResult {
    let global_idx = GlobalIdx::parse(i)?;
    let global = match ctxt.module.globals.get(global_idx as usize) {
        Some(g) => g,
        None => {
            ctxt.poison::<()>(ValidationError::Msg(format!(
                "global variable with index {global_idx} not found",
            )));
            return Ok(());
        }
    };

    let global_type = match &global.r#type {
        GlobalType::Const(t) => t,
        GlobalType::Mut(t) => t,
    };
    let out = ctxt.create_var(*global_type);
    o.write_global_get(GlobalGetInstruction {
        out1: out.id,
        global_idx,
    });
    ctxt.push_var(out);
    Ok(())
}

pub(crate) fn local_set(
    ctxt: &mut Context,
    i: &mut WasmBinaryReader,
    o: &mut dyn InstructionConsumer,
) -> ParseResult {
    let local_idx = LocalIdx::parse(i)?;
    let local_ty = match ctxt.locals.get(local_idx as usize) {
        Some(local_ty) => *local_ty,
        None => {
            ctxt.poison::<()>(ValidationError::Msg(format!(
                "local variable with index {local_idx} not found",
            )));
            return Ok(());
        }
    };

    let value = ctxt.pop_var_with_type(local_ty);
    if value.type_ != local_ty {
        ctxt.poison(ValidationError::Msg(format!(
            "stack value type does not match local {} type: {:?} vs {:?}",
            local_idx, value.type_, local_ty
        )))
    } else {
        o.write_local_set(LocalSetInstruction {
            local_idx,
            in1: value.id,
        });
    }
    Ok(())
}

pub(crate) fn global_set(
    ctxt: &mut Context,
    i: &mut WasmBinaryReader,
    o: &mut dyn InstructionConsumer,
) -> ParseResult {
    let global_idx = GlobalIdx::parse(i)?;
    let global = match ctxt.module.globals.get(global_idx as usize) {
        Some(g) => g,
        None => {
            ctxt.poison::<()>(ValidationError::Msg(format!(
                "global variable with index {global_idx} not found",
            )));
            return Ok(());
        }
    };

    let global_type = match &global.r#type {
        GlobalType::Const(_) => {
            ctxt.poison::<()>(ValidationError::Msg(format!(
                "cannot set const global {global_idx}",
            )));
            return Ok(());
        }
        GlobalType::Mut(t) => *t,
    };

    let value = ctxt.pop_var_with_type(global_type);
    if value.type_ != global_type {
        ctxt.poison(ValidationError::Msg(format!(
            "stack value type does not match global {} type: {:?} vs {:?}",
            global_idx, value.type_, global_type
        )))
    }

    o.write_global_set(GlobalSetInstruction {
        global_idx,
        in1: value.id,
    });
    Ok(())
}

pub(crate) fn local_tee(
    ctxt: &mut Context,
    i: &mut WasmBinaryReader,
    o: &mut dyn InstructionConsumer,
) -> ParseResult {
    let local_idx = LocalIdx::parse(i)?;
    let local_ty = match ctxt.locals.get(local_idx as usize) {
        Some(local_ty) => *local_ty,
        None => {
            ctxt.poison::<()>(ValidationError::Msg(format!(
                "local variable with index {local_idx} not found",
            )));
            return Ok(());
        }
    };

    let in_stack_var = ctxt.pop_var_with_type(local_ty);
    if in_stack_var.type_ != local_ty {
        ctxt.poison(ValidationError::Msg(format!(
            "stack value type does not match local {} type: {:?} vs {:?}",
            local_idx, in_stack_var.type_, local_ty
        )))
    }

    let out_stack_var = ctxt.create_var(local_ty);

    o.write_local_tee(LocalTeeInstruction {
        local_idx,
        in1: in_stack_var.id,
        out1: out_stack_var.id,
    });

    ctxt.push_var(out_stack_var);
    Ok(())
}
