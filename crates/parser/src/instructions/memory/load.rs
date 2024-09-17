use super::*;

fn parse_load(
    ctxt: &mut Context,
    i: &mut WasmBinaryReader,
    o: &mut InstructionEncoder,
    out_type: NumType,
    operation: LoadOp,
) -> ParseResult {
    let memarg = MemArg::parse(i)?;
    let natural_alignment = match operation {
        LoadOp::INNLoad | LoadOp::FNNLoad => match out_type {
            NumType::I32 | NumType::F32 => 4,
            NumType::I64 | NumType::F64 => 8,
        },
        LoadOp::INNLoad8S | LoadOp::INNLoad8U => 1,
        LoadOp::INNLoad16S | LoadOp::INNLoad16U => 2,
        LoadOp::INNLoad32S | LoadOp::INNLoad32U => 4,
    };
    if 2_u32.pow(memarg.align) > natural_alignment {
        return Err(ParserError::AlignmentLargerThanNatural);
    }

    let in_ = ctxt.pop_var_with_type(&ValType::i32());
    let out = ctxt.create_var(ValType::Number(out_type));
    o.write(LoadInstruction {
        memarg,
        out1: out.id,
        out1_type: out_type,
        addr: in_.id,
        operation,
    });
    ctxt.push_var(out);
    Ok(())
}

#[rustfmt::skip]
mod load_specializations {
    use super::*;
    pub(crate) fn i32_load(c: &mut C, i: &mut I, o: &mut O) -> PR {parse_load(c, i, o, NumType::I32, LoadOp::INNLoad)}
    pub(crate) fn i64_load(c: &mut C, i: &mut I, o: &mut O) -> PR {parse_load(c, i, o, NumType::I64, LoadOp::INNLoad)}
    pub(crate) fn f32_load(c: &mut C, i: &mut I, o: &mut O) -> PR {parse_load(c, i, o, NumType::F32, LoadOp::FNNLoad)}
    pub(crate) fn f64_load(c: &mut C, i: &mut I, o: &mut O) -> PR {parse_load(c, i, o, NumType::F64, LoadOp::FNNLoad)}

    pub(crate) fn i32_load8_s(c: &mut C, i: &mut I, o: &mut O) -> PR {parse_load(c, i, o, NumType::I32, LoadOp::INNLoad8S)}
    pub(crate) fn i32_load8_u(c: &mut C, i: &mut I, o: &mut O) -> PR {parse_load(c, i, o, NumType::I32, LoadOp::INNLoad8U)}
    pub(crate) fn i32_load16_s(c: &mut C, i: &mut I, o: &mut O) -> PR {parse_load(c, i, o, NumType::I32, LoadOp::INNLoad16S)}
    pub(crate) fn i32_load16_u(c: &mut C, i: &mut I, o: &mut O) -> PR {parse_load(c, i, o, NumType::I32, LoadOp::INNLoad16U)}

    pub(crate) fn i64_load8_s(c: &mut C, i: &mut I, o: &mut O) -> PR {parse_load(c, i, o, NumType::I64, LoadOp::INNLoad8S)}
    pub(crate) fn i64_load8_u(c: &mut C, i: &mut I, o: &mut O) -> PR {parse_load(c, i, o, NumType::I64, LoadOp::INNLoad8U)}
    pub(crate) fn i64_load16_s(c: &mut C, i: &mut I, o: &mut O) -> PR {parse_load(c, i, o, NumType::I64, LoadOp::INNLoad16S)}
    pub(crate) fn i64_load16_u(c: &mut C, i: &mut I, o: &mut O) -> PR {parse_load(c, i, o, NumType::I64, LoadOp::INNLoad16U)}
    pub(crate) fn i64_load32_s(c: &mut C, i: &mut I, o: &mut O) -> PR {parse_load(c, i, o, NumType::I64, LoadOp::INNLoad32S)}
    pub(crate) fn i64_load32_u(c: &mut C, i: &mut I, o: &mut O) -> PR {parse_load(c, i, o, NumType::I64, LoadOp::INNLoad32U)}
}
use module::{instructions::LoadInstruction, objects::memory::MemArg};
pub(crate) use load_specializations::*;
