use super::*;
use wasm_types::*;

fn i_arith(ctxt: &mut C, o: &mut dyn InstructionConsumer, op: IBinaryOp, type_: NumType) -> PR {
    let rhs = ctxt.pop_var_with_type(ValType::Number(type_));
    let lhs = ctxt.pop_var_with_type(ValType::Number(type_));
    let out = ctxt.create_var(ValType::Number(type_));
    o.write_ibinary(IBinaryInstruction {
        types: type_,
        op,
        lhs: lhs.id,
        rhs: rhs.id,
        out1: out.id,
    });
    ctxt.push_var(out);
    Ok(())
}

fn f_arith(ctxt: &mut C, o: &mut dyn InstructionConsumer, op: FBinaryOp, type_: NumType) -> PR {
    let rhs = ctxt.pop_var_with_type(ValType::Number(type_));
    let lhs = ctxt.pop_var_with_type(ValType::Number(type_));
    let out = ctxt.create_var(ValType::Number(type_));
    o.write_fbinary(FBinaryInstruction {
        types: type_,
        op,
        lhs: lhs.id,
        rhs: rhs.id,
        out1: out.id,
    });
    ctxt.push_var(out);
    Ok(())
}

#[rustfmt::skip]
mod specializations {
    use super::*;
    pub(crate) fn i32_add(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {i_arith(ctxt, o, IBinaryOp::Add, NumType::I32)}
    pub(crate) fn i32_sub(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {i_arith(ctxt, o, IBinaryOp::Sub, NumType::I32)}
    pub(crate) fn i32_mul(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {i_arith(ctxt, o, IBinaryOp::Mul, NumType::I32)}
    pub(crate) fn i32_div_s(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {i_arith(ctxt, o, IBinaryOp::DivS, NumType::I32)}
    pub(crate) fn i32_div_u(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {i_arith(ctxt, o, IBinaryOp::DivU, NumType::I32)}
    pub(crate) fn i32_rem_s(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {i_arith(ctxt, o, IBinaryOp::RemS, NumType::I32)}
    pub(crate) fn i32_rem_u(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {i_arith(ctxt, o, IBinaryOp::RemU, NumType::I32)}
    pub(crate) fn i32_and(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {i_arith(ctxt, o, IBinaryOp::And, NumType::I32)}
    pub(crate) fn i32_or(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {i_arith(ctxt, o, IBinaryOp::Or, NumType::I32)}
    pub(crate) fn i32_xor(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {i_arith(ctxt, o, IBinaryOp::Xor, NumType::I32)}
    pub(crate) fn i32_shl(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {i_arith(ctxt, o, IBinaryOp::Shl, NumType::I32)}
    pub(crate) fn i32_shr_s(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {i_arith(ctxt, o, IBinaryOp::ShrS, NumType::I32)}
    pub(crate) fn i32_shr_u(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {i_arith(ctxt, o, IBinaryOp::ShrU, NumType::I32)}
    pub(crate) fn i32_rotl(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {i_arith(ctxt, o, IBinaryOp::Rotl, NumType::I32)}
    pub(crate) fn i32_rotr(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {i_arith(ctxt, o, IBinaryOp::Rotr, NumType::I32)}

    pub(crate) fn i64_add(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {i_arith(ctxt, o, IBinaryOp::Add, NumType::I64)}
    pub(crate) fn i64_sub(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {i_arith(ctxt, o, IBinaryOp::Sub, NumType::I64)}
    pub(crate) fn i64_mul(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {i_arith(ctxt, o, IBinaryOp::Mul, NumType::I64)}
    pub(crate) fn i64_div_s(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {i_arith(ctxt, o, IBinaryOp::DivS, NumType::I64)}
    pub(crate) fn i64_div_u(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {i_arith(ctxt, o, IBinaryOp::DivU, NumType::I64)}
    pub(crate) fn i64_rem_s(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {i_arith(ctxt, o, IBinaryOp::RemS, NumType::I64)}
    pub(crate) fn i64_rem_u(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {i_arith(ctxt, o, IBinaryOp::RemU, NumType::I64)}
    pub(crate) fn i64_and(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {i_arith(ctxt, o, IBinaryOp::And, NumType::I64)}
    pub(crate) fn i64_or(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {i_arith(ctxt, o, IBinaryOp::Or, NumType::I64)}
    pub(crate) fn i64_xor(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {i_arith(ctxt, o, IBinaryOp::Xor, NumType::I64)}
    pub(crate) fn i64_shl(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {i_arith(ctxt, o, IBinaryOp::Shl, NumType::I64)}
    pub(crate) fn i64_shr_s(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {i_arith(ctxt, o, IBinaryOp::ShrS, NumType::I64)}
    pub(crate) fn i64_shr_u(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {i_arith(ctxt, o, IBinaryOp::ShrU, NumType::I64)}
    pub(crate) fn i64_rotl(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {i_arith(ctxt, o, IBinaryOp::Rotl, NumType::I64)}
    pub(crate) fn i64_rotr(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {i_arith(ctxt, o, IBinaryOp::Rotr, NumType::I64)}

    pub(crate) fn f32_add(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {f_arith(ctxt, o, FBinaryOp::Add, NumType::F32)}
    pub(crate) fn f32_sub(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {f_arith(ctxt, o, FBinaryOp::Sub, NumType::F32)}
    pub(crate) fn f32_mul(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {f_arith(ctxt, o, FBinaryOp::Mul, NumType::F32)}
    pub(crate) fn f32_div(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {f_arith(ctxt, o, FBinaryOp::Div, NumType::F32)}
    pub(crate) fn f32_min(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {f_arith(ctxt, o, FBinaryOp::Min, NumType::F32)}
    pub(crate) fn f32_max(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {f_arith(ctxt, o, FBinaryOp::Max, NumType::F32)}
    pub(crate) fn f32_copysign(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {f_arith(ctxt, o, FBinaryOp::Copysign, NumType::F32)}

    pub(crate) fn f64_add(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {f_arith(ctxt, o, FBinaryOp::Add, NumType::F64)}
    pub(crate) fn f64_sub(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {f_arith(ctxt, o, FBinaryOp::Sub, NumType::F64)}
    pub(crate) fn f64_mul(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {f_arith(ctxt, o, FBinaryOp::Mul, NumType::F64)}
    pub(crate) fn f64_div(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {f_arith(ctxt, o, FBinaryOp::Div, NumType::F64)}
    pub(crate) fn f64_min(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {f_arith(ctxt, o, FBinaryOp::Min, NumType::F64)}
    pub(crate) fn f64_max(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {f_arith(ctxt, o, FBinaryOp::Max, NumType::F64)}
    pub(crate) fn f64_copysign(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {f_arith(ctxt, o, FBinaryOp::Copysign, NumType::F64)}
}
pub(crate) use specializations::*;
