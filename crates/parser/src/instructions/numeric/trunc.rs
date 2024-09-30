use super::*;
use wasm_types::*;

pub(crate) fn parse_trunc(
    ctxt: &mut Context,
    o: &mut dyn InstructionConsumer,
    input_type: NumType,
    out_type: NumType,
    signed: bool,
) -> ParseResult {
    let in_ = ctxt.pop_var_with_type(ValType::Number(input_type));
    let out = ctxt.create_var(ValType::Number(out_type));
    o.write_trunc(TruncInstruction {
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
mod trunc_specializations {
    use super::*;

    pub(crate) fn i32_trunc_f32_s(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {parse_trunc(ctxt, o, NumType::F32, NumType::I32, true)}
    pub(crate) fn i32_trunc_f32_u(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {parse_trunc(ctxt, o, NumType::F32, NumType::I32, false)}
    pub(crate) fn i32_trunc_f64_s(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {parse_trunc(ctxt, o, NumType::F64, NumType::I32, true)}
    pub(crate) fn i32_trunc_f64_u(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {parse_trunc(ctxt, o, NumType::F64, NumType::I32, false)}

    pub(crate) fn i64_trunc_f32_s(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {parse_trunc(ctxt, o, NumType::F32, NumType::I64, true)}
    pub(crate) fn i64_trunc_f32_u(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {parse_trunc(ctxt, o, NumType::F32, NumType::I64, false)}
    pub(crate) fn i64_trunc_f64_s(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {parse_trunc(ctxt, o, NumType::F64, NumType::I64, true)}
    pub(crate) fn i64_trunc_f64_u(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {parse_trunc(ctxt, o, NumType::F64, NumType::I64, false)}
}
pub(crate) use trunc_specializations::*;

fn parse_trunc_sat(
    ctxt: &mut C,
    o: &mut dyn InstructionConsumer,
    in_type: NumType,
    out_type: NumType,
    signed: bool,
) -> PR {
    let in1 = ctxt.pop_var_with_type(ValType::Number(in_type));
    let out1 = ctxt.create_var(ValType::Number(out_type));
    o.write_trunc_saturation(TruncSaturationInstruction {
        in1: in1.id,
        out1: out1.id,
        in1_type: in_type,
        out1_type: out_type,
        signed,
    });
    ctxt.push_var(out1);
    Ok(())
}

#[rustfmt::skip]
mod specializations {
    use super::*;
    pub(crate) fn i32_trunc_sat_f32_s(c: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {parse_trunc_sat(c, o, NumType::F32, NumType::I32, true)}
    pub(crate) fn i32_trunc_sat_f32_u(c: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {parse_trunc_sat(c, o, NumType::F32, NumType::I32, false)}
    pub(crate) fn i32_trunc_sat_f64_s(c: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {parse_trunc_sat(c, o, NumType::F64, NumType::I32, true)}
    pub(crate) fn i32_trunc_sat_f64_u(c: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {parse_trunc_sat(c, o, NumType::F64, NumType::I32, false)}
    pub(crate) fn i64_trunc_sat_f32_s(c: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {parse_trunc_sat(c, o, NumType::F32, NumType::I64, true)}
    pub(crate) fn i64_trunc_sat_f32_u(c: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {parse_trunc_sat(c, o, NumType::F32, NumType::I64, false)}
    pub(crate) fn i64_trunc_sat_f64_s(c: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {parse_trunc_sat(c, o, NumType::F64, NumType::I64, true)}
    pub(crate) fn i64_trunc_sat_f64_u(c: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {parse_trunc_sat(c, o, NumType::F64, NumType::I64, false)}
}
pub(crate) use specializations::*;
