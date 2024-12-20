use super::*;
use wasm_types::*;

fn i_arith(ctxt: &mut C, o: &mut dyn InstructionConsumer, op: ITestOp, type_: NumType) -> PR {
    let in_ = ctxt.pop_var_with_type(ValType::Number(type_));
    let out = ctxt.create_var(ValType::i32());
    o.write_test(ITestInstruction {
        input_type: type_,
        op,
        in1: in_.id,
        out1: out.id,
    });
    ctxt.push_var(out);
    Ok(())
}

#[rustfmt::skip]
mod specializations {
    use super::*;
    pub(crate) fn i32_eqz(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {i_arith(ctxt, o, ITestOp::Eqz, NumType::I32)}
    pub(crate) fn i64_eqz(ctxt: &mut C, _: &mut I, o: &mut dyn InstructionConsumer) -> PR {i_arith(ctxt, o, ITestOp::Eqz, NumType::I64)}
}
pub(crate) use specializations::*;
