use super::*;
use wasm_types::*;

#[derive(Debug, Clone)]
pub(crate) struct TruncSaturationInstruction {
    in1: VariableID,
    out1: VariableID,
    in1_type: NumType,
    out1_type: NumType,
    signed: bool,
}

impl Instruction for TruncSaturationInstruction {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.write_variable(self.in1);
        o.write_variable(self.out1);
        o.write_value_type(ValType::Number(self.in1_type));
        o.write_value_type(ValType::Number(self.out1_type));
        o.write_immediate(self.signed as u8);
    }

    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let in1 = i.read_variable()?;
        let out1 = i.read_variable()?;
        let in1_type = extract_numtype!(i.read_value_type()?);
        let out1_type = extract_numtype!(i.read_value_type()?);
        let signed = i.read_immediate::<u8>()? != 0;
        Ok(TruncSaturationInstruction {
            in1,
            out1,
            in1_type,
            out1_type,
            signed,
        })
    }
}

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
