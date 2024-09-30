use super::*;
use crate::parsable::Parse;
use wasm_types::*;

pub(crate) fn drop(
    ctxt: &mut Context,
    _: &mut WasmBinaryReader,
    _: &mut dyn InstructionConsumer,
) -> ParseResult {
    ctxt.pop_var();
    Ok(())
}

pub(crate) fn select_numeric(
    ctxt: &mut Context,
    _: &mut WasmBinaryReader,
    o: &mut dyn InstructionConsumer,
) -> ParseResult {
    let select_val = ctxt.pop_var_with_type(ValType::i32());

    let val2 = ctxt.pop_var();
    let val1 = ctxt.pop_var();
    if val1.type_ != val2.type_ {
        ctxt.poison(ValidationError::Msg(format!(
            "type mismatch for select: {:?} != {:?}",
            val1.type_, val2.type_
        )))
    }
    match &val1.type_ {
        ValType::Number(_) | ValType::VecType => {}
        _ => ctxt.poison(ValidationError::Msg(format!(
            "invalid type for numeric select: {:?}",
            val1.type_
        ))),
    }

    let out = ctxt.create_var(val1.type_);
    o.write_select(SelectInstruction {
        input_vals: [val1.id, val2.id],
        select_val: select_val.id,
        out1: out.id,
    });
    ctxt.push_var(out);
    Ok(())
}

pub(crate) fn select_generic(
    ctxt: &mut Context,
    i: &mut WasmBinaryReader,
    o: &mut dyn InstructionConsumer,
) -> ParseResult {
    let num_val_types = i.read_leb128::<u32>()?;
    if num_val_types != 1 {
        ctxt.poison(ValidationError::Msg(format!(
            "invalid number of val types for select: {num_val_types} != 1"
        )))
    }
    let val_type = ValType::parse(i)?;
    let select_val = ctxt.pop_var_with_type(ValType::i32());

    let val2 = ctxt.pop_var_with_type(val_type);
    let val1 = ctxt.pop_var_with_type(val_type);
    let out = ctxt.create_var(val_type);
    o.write_select(SelectInstruction {
        input_vals: [val1.id, val2.id],
        select_val: select_val.id,
        out1: out.id,
    });
    ctxt.push_var(out);
    Ok(())
}
