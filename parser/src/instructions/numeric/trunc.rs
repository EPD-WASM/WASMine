use super::*;
use wasm_types::*;

fn parse_trunc(ctxt: &mut C, o: &mut O, in_type: NumType, out_type: NumType, signed: bool) -> PR {
    let in1 = ctxt.pop_var_with_type(&ValType::Number(in_type));
    let out1 = ctxt.create_var(ValType::Number(out_type));
    o.write(TruncSaturationInstruction {
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
    pub(crate) fn i32_trunc_sat_f32_s(c: &mut C, _: &mut I, o: &mut O) -> PR {parse_trunc(c, o, NumType::F32, NumType::I32, true)}
    pub(crate) fn i32_trunc_sat_f32_u(c: &mut C, _: &mut I, o: &mut O) -> PR {parse_trunc(c, o, NumType::F32, NumType::I32, false)}
    pub(crate) fn i32_trunc_sat_f64_s(c: &mut C, _: &mut I, o: &mut O) -> PR {parse_trunc(c, o, NumType::F64, NumType::I32, true)}
    pub(crate) fn i32_trunc_sat_f64_u(c: &mut C, _: &mut I, o: &mut O) -> PR {parse_trunc(c, o, NumType::F64, NumType::I32, false)}
    pub(crate) fn i64_trunc_sat_f32_s(c: &mut C, _: &mut I, o: &mut O) -> PR {parse_trunc(c, o, NumType::F32, NumType::I64, true)}
    pub(crate) fn i64_trunc_sat_f32_u(c: &mut C, _: &mut I, o: &mut O) -> PR {parse_trunc(c, o, NumType::F32, NumType::I64, false)}
    pub(crate) fn i64_trunc_sat_f64_s(c: &mut C, _: &mut I, o: &mut O) -> PR {parse_trunc(c, o, NumType::F64, NumType::I64, true)}
    pub(crate) fn i64_trunc_sat_f64_u(c: &mut C, _: &mut I, o: &mut O) -> PR {parse_trunc(c, o, NumType::F64, NumType::I64, false)}
}
pub(crate) use specializations::*;
