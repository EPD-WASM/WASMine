use super::*;

fn parse_store(
    ctxt: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
    input_type: NumType,
    operation: StoreOp,
) -> ParseResult {
    let memarg = MemArg::parse(i)?;
    let value_in = ctxt.pop_var_with_type(&ValType::Number(input_type));
    let addr_in = ctxt.pop_var_with_type(&ValType::Number(NumType::I32));
    o.write(StoreInstruction {
        memarg,
        addr_in: addr_in.id,
        value_in: value_in.id,
        operation,
    });
    Ok(())
}

#[rustfmt::skip]
mod store_specializations {
    use super::*;
    pub(crate) fn i32_store(c: &mut C, i: &mut I, o: &mut O) -> PR {parse_store(c, i, o, NumType::I32, StoreOp::INNStore)}
    pub(crate) fn i64_store(c: &mut C, i: &mut I, o: &mut O) -> PR {parse_store(c, i, o, NumType::I64, StoreOp::INNStore)}
    pub(crate) fn f32_store(c: &mut C, i: &mut I, o: &mut O) -> PR {parse_store(c, i, o, NumType::F32, StoreOp::FNNStore)}
    pub(crate) fn f64_store(c: &mut C, i: &mut I, o: &mut O) -> PR {parse_store(c, i, o, NumType::F64, StoreOp::FNNStore)}
    pub(crate) fn i32_store8(c: &mut C, i: &mut I, o: &mut O) -> PR {parse_store(c, i, o, NumType::I32, StoreOp::INNStore8)}
    pub(crate) fn i32_store16(c: &mut C, i: &mut I, o: &mut O) -> PR {parse_store(c, i, o, NumType::I32, StoreOp::INNStore16)}
    pub(crate) fn i64_store8(c: &mut C, i: &mut I, o: &mut O) -> PR {parse_store(c, i, o, NumType::I64, StoreOp::INNStore8)}
    pub(crate) fn i64_store16(c: &mut C, i: &mut I, o: &mut O) -> PR {parse_store(c, i, o, NumType::I64, StoreOp::INNStore16)}
    pub(crate) fn i64_store32(c: &mut C, i: &mut I, o: &mut O) -> PR {parse_store(c, i, o, NumType::I64, StoreOp::INNStore32)}
}
pub(crate) use store_specializations::*;
