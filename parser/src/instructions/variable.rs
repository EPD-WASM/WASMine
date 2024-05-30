use super::*;
use crate::parsable::Parse;
use wasm_types::*;

pub(crate) fn local_get(
    ctxt: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let local_idx = LocalIdx::parse(i)?;
    let var = match ctxt.func.locals.get(local_idx as usize) {
        Some(v) => v,
        None => {
            ctxt.poison::<()>(ValidationError::Msg(format!(
                "local variable with index {} not found",
                local_idx
            )));
            return Ok(());
        }
    };

    let out = ctxt.create_var(var.type_);
    o.write(LocalGetInstruction {
        out1: out.id,
        local_idx,
    });
    ctxt.push_var(out);
    Ok(())
}

pub(crate) fn global_get(
    ctxt: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let global_idx = GlobalIdx::parse(i)?;
    let global = match ctxt.module.globals.get(global_idx as usize) {
        Some(g) => g,
        None => {
            ctxt.poison::<()>(ValidationError::Msg(format!(
                "global variable with index {} not found",
                global_idx
            )));
            return Ok(());
        }
    };

    let global_type = match &global.r#type {
        GlobalType::Const(t) => t,
        GlobalType::Mut(t) => t,
    };
    let out = ctxt.create_var(*global_type);
    o.write(GlobalGetInstruction {
        out1: out.id,
        global_idx,
    });
    ctxt.push_var(out);
    Ok(())
}

pub(crate) fn local_set(
    ctxt: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let local_idx = LocalIdx::parse(i)?;
    let var = match ctxt.func.locals.get(local_idx as usize) {
        Some(v) => v,
        None => {
            ctxt.poison::<()>(ValidationError::Msg(format!(
                "local variable with index {} not found",
                local_idx
            )));
            return Ok(());
        }
    };

    let value = ctxt.pop_var_with_type(&var.type_);
    if value.type_ != var.type_ {
        ctxt.poison(ValidationError::Msg(format!(
            "stack value type does not match local {} type: {:?} vs {:?}",
            local_idx, value.type_, var.type_
        )))
    }

    o.write(LocalSetInstruction {
        local_idx,
        in1: value.id,
    });
    Ok(())
}

pub(crate) fn global_set(
    ctxt: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let global_idx = GlobalIdx::parse(i)?;
    let global = match ctxt.module.globals.get(global_idx as usize) {
        Some(g) => g,
        None => {
            ctxt.poison::<()>(ValidationError::Msg(format!(
                "global variable with index {} not found",
                global_idx
            )));
            return Ok(());
        }
    };

    let global_type = match &global.r#type {
        GlobalType::Const(_) => {
            ctxt.poison::<()>(ValidationError::Msg(format!(
                "cannot set const global {}",
                global_idx
            )));
            return Ok(());
        }
        GlobalType::Mut(t) => t,
    };

    let value = ctxt.pop_var_with_type(global_type);
    if value.type_ != *global_type {
        ctxt.poison(ValidationError::Msg(format!(
            "stack value type does not match global {} type: {:?} vs {:?}",
            global_idx, value.type_, global_type
        )))
    }

    o.write(GlobalSetInstruction {
        global_idx,
        in1: value.id,
    });
    Ok(())
}

pub(crate) fn local_tee(
    ctxt: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let local_idx = LocalIdx::parse(i)?;
    let local_var = match ctxt.func.locals.get(local_idx as usize) {
        Some(v) => v,
        None => {
            ctxt.poison::<()>(ValidationError::Msg(format!(
                "local variable with index {} not found",
                local_idx
            )));
            return Ok(());
        }
    };

    let in_stack_var = ctxt.pop_var_with_type(&local_var.type_);
    if in_stack_var.type_ != local_var.type_ {
        ctxt.poison(ValidationError::Msg(format!(
            "stack value type does not match local {} type: {:?} vs {:?}",
            local_idx, in_stack_var.type_, local_var.type_
        )))
    }

    let out_stack_var = ctxt.create_var(local_var.type_);

    o.write(TeeLocalInstruction {
        local_idx,
        in1: in_stack_var.id,
        out1: out_stack_var.id,
    });

    ctxt.push_var(out_stack_var);
    Ok(())
}
