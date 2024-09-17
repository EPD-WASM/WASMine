use super::*;
use crate::parsable::Parse;

pub(crate) fn ref_is_null(
    ctxt: &mut Context,
    _: &mut WasmBinaryReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let val = ctxt.pop_var();
    match val.type_ {
        ValType::Reference(_) => {}
        _ => ctxt.poison(ValidationError::Msg(
            "ref.is_null expects a reference type on stack".into(),
        )),
    }
    let out = ctxt.create_var(ValType::i32());
    o.write(ReferenceIsNullInstruction {
        in1: val.id,
        in1_type: val.type_,
        out1: out.id,
    });
    ctxt.push_var(out);
    Ok(())
}

pub(crate) fn ref_null(
    ctxt: &mut Context,
    i: &mut WasmBinaryReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let ref_type = RefType::parse(i)?;
    let out = ctxt.create_var(ValType::Reference(ref_type));
    o.write(ReferenceNullInstruction {
        out1: out.id,
        out1_type: ref_type,
    });
    ctxt.push_var(out);
    Ok(())
}

pub(crate) fn ref_func(
    ctxt: &mut Context,
    i: &mut WasmBinaryReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let func_idx = FuncIdx::parse(i)?;
    let out = ctxt.create_var(ValType::funcref());
    o.write(ReferenceFunctionInstruction {
        out1: out.id,
        func_idx,
    });
    ctxt.push_var(out);
    Ok(())
}
