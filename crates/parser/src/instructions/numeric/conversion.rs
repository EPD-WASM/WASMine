use super::*;
use wasm_types::*;

pub(crate) fn i32_wrap_i64(
    ctxt: &mut Context,
    _: &mut WasmBinaryReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let in_ = ctxt.pop_var_with_type(&ValType::i64());
    let out = ctxt.create_var(ValType::i32());
    o.write(WrapInstruction {
        in1: in_.id,
        out1: out.id,
    });
    ctxt.push_var(out);
    Ok(())
}

pub(crate) fn parse_convert(
    ctxt: &mut Context,
    o: &mut InstructionEncoder,
    input_type: NumType,
    out_type: NumType,
    signed: bool,
) -> ParseResult {
    let in_ = ctxt.pop_var_with_type(&ValType::Number(input_type));
    let out = ctxt.create_var(ValType::Number(out_type));
    o.write(ConvertInstruction {
        in1: in_.id,
        in1_type: input_type,
        out1: out.id,
        out1_type: out_type,
        signed,
    });
    ctxt.push_var(out);
    Ok(())
}

#[rustfmt::skip]
mod convert_specializations {
    use super::*;

    pub(crate) fn f32_convert_i32_s(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {parse_convert(ctxt, o, NumType::I32, NumType::F32, true)}
    pub(crate) fn f32_convert_i32_u(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {parse_convert(ctxt, o, NumType::I32, NumType::F32, false)}
    pub(crate) fn f32_convert_i64_s(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {parse_convert(ctxt, o, NumType::I64, NumType::F32, true)}
    pub(crate) fn f32_convert_i64_u(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {parse_convert(ctxt, o, NumType::I64, NumType::F32, false)}

    pub(crate) fn f64_convert_i32_s(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {parse_convert(ctxt, o, NumType::I32, NumType::F64, true)}
    pub(crate) fn f64_convert_i32_u(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {parse_convert(ctxt, o, NumType::I32, NumType::F64, false)}
    pub(crate) fn f64_convert_i64_s(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {parse_convert(ctxt, o, NumType::I64, NumType::F64, true)}
    pub(crate) fn f64_convert_i64_u(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {parse_convert(ctxt, o, NumType::I64, NumType::F64, false)}
}
pub(crate) use convert_specializations::*;

pub(crate) fn parse_reinterpret(
    ctxt: &mut Context,
    _: &mut WasmBinaryReader,
    o: &mut InstructionEncoder,
    input_type: NumType,
    out_type: NumType,
) -> ParseResult {
    let in_ = ctxt.pop_var_with_type(&ValType::Number(input_type));
    let out = ctxt.create_var(ValType::Number(out_type));
    o.write(ReinterpretInstruction {
        in1: in_.id,
        in1_type: input_type,
        out1: out.id,
        out1_type: out_type,
    });
    ctxt.push_var(out);
    Ok(())
}

#[rustfmt::skip]
mod reinterpret_specializations {
    use super::*;

    pub(crate) fn i32_reinterpret_f32(ctxt: &mut C, i: &mut I, o: &mut O) -> PR {parse_reinterpret(ctxt, i, o, NumType::F32, NumType::I32)}
    pub(crate) fn i64_reinterpret_f64(ctxt: &mut C, i: &mut I, o: &mut O) -> PR {parse_reinterpret(ctxt, i, o, NumType::F64, NumType::I64)}
    pub(crate) fn f32_reinterpret_i32(ctxt: &mut C, i: &mut I, o: &mut O) -> PR {parse_reinterpret(ctxt, i, o, NumType::I32, NumType::F32)}
    pub(crate) fn f64_reinterpret_i64(ctxt: &mut C, i: &mut I, o: &mut O) -> PR {parse_reinterpret(ctxt, i, o, NumType::I64, NumType::F64)}
}
pub(crate) use reinterpret_specializations::*;

pub(crate) fn parse_extend(
    ctxt: &mut Context,
    _: &mut WasmBinaryReader,
    o: &mut InstructionEncoder,
    input_size: u8,
    out_type: NumType,
) -> ParseResult {
    let in_ = ctxt.pop_var_with_type(&ValType::Number(out_type));
    let out = ctxt.create_var(ValType::Number(out_type));
    o.write(ExtendBitsInstruction {
        in1: in_.id,
        in1_type: out_type,
        input_size,
        out1: out.id,
        out1_type: out_type,
    });
    ctxt.push_var(out);
    Ok(())
}

#[rustfmt::skip]
mod extend_specializations {
    use super::*;

    pub(crate) fn i32_extend8_s(ctxt: &mut C, i: &mut I, o: &mut O) -> PR {parse_extend(ctxt, i, o, 8, NumType::I32)}
    pub(crate) fn i32_extend16_s(ctxt: &mut C, i: &mut I, o: &mut O) -> PR {parse_extend(ctxt, i, o, 16, NumType::I32)}
    pub(crate) fn i64_extend8_s(ctxt: &mut C, i: &mut I, o: &mut O) -> PR {parse_extend(ctxt, i, o, 8, NumType::I64)}
    pub(crate) fn i64_extend16_s(ctxt: &mut C, i: &mut I, o: &mut O) -> PR {parse_extend(ctxt, i, o, 16, NumType::I64)}
    pub(crate) fn i64_extend32_s(ctxt: &mut C, i: &mut I, o: &mut O) -> PR {parse_extend(ctxt, i, o, 32, NumType::I64)}
}
pub(crate) use extend_specializations::*;

pub(crate) fn parse_extend_type(
    ctxt: &mut Context,
    _: &mut WasmBinaryReader,
    o: &mut InstructionEncoder,
    signed: bool,
) -> ParseResult {
    let in_ = ctxt.pop_var_with_type(&ValType::i32());
    let out = ctxt.create_var(ValType::i64());
    o.write(ExtendTypeInstruction {
        signed,
        out1: out.id,
        in1: in_.id,
    });
    ctxt.push_var(out);
    Ok(())
}

#[rustfmt::skip]
mod extend_type_specializations {
    use super::*;

    pub(crate) fn i64_extend_i32_s(ctxt: &mut C, i: &mut I, o: &mut O) -> PR {parse_extend_type(ctxt, i, o, true)}
    pub(crate) fn i64_extend_i32_u(ctxt: &mut C, i: &mut I, o: &mut O) -> PR {parse_extend_type(ctxt, i, o, false)}
}
pub(crate) use extend_type_specializations::*;

pub(crate) fn f32_demote_f64(
    ctxt: &mut Context,
    _: &mut WasmBinaryReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let in_ = ctxt.pop_var_with_type(&ValType::f64());
    let out = ctxt.create_var(ValType::f32());
    o.write(DemoteInstruction {
        in1: in_.id,
        out1: out.id,
    });
    ctxt.push_var(out);
    Ok(())
}

pub(crate) fn f64_promote_f32(
    ctxt: &mut Context,
    _: &mut WasmBinaryReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let in_ = ctxt.pop_var_with_type(&ValType::f32());
    let out = ctxt.create_var(ValType::f64());
    o.write(PromoteInstruction {
        in1: in_.id,
        out1: out.id,
    });
    ctxt.push_var(out);
    Ok(())
}
