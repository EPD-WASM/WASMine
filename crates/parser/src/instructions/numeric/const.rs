use super::*;
use wasm_types::*;

pub(crate) fn i32_const_i32(ctxt: &mut C, i: &mut I, o: &mut O) -> PR {
    let imm = i.read_leb128::<i32>()?;
    let imm_var = ctxt.create_var(ValType::i32());
    let const_instr = Constant {
        imm: imm.into(),
        out1: imm_var.id,
        out1_type: NumType::I32,
    };
    o.write(const_instr);
    ctxt.push_var(imm_var);
    Ok(())
}

pub(crate) fn i64_const_i64(ctxt: &mut C, i: &mut I, o: &mut O) -> PR {
    let imm = i.read_leb128::<i64>()?;
    let imm_var = ctxt.create_var(ValType::i64());
    let const_instr = Constant {
        imm: imm.into(),
        out1: imm_var.id,
        out1_type: NumType::I64,
    };
    o.write(const_instr);
    ctxt.push_var(imm_var);
    Ok(())
}

pub(crate) fn f32_const_f32(ctxt: &mut C, i: &mut I, o: &mut O) -> PR {
    let imm = i.read_f32()?;
    let imm_var = ctxt.create_var(ValType::f32());
    let const_instr = Constant {
        imm: imm.into(),
        out1: imm_var.id,
        out1_type: NumType::F32,
    };
    o.write(const_instr);
    ctxt.push_var(imm_var);
    Ok(())
}

pub(crate) fn f64_const_f64(ctxt: &mut C, i: &mut I, o: &mut O) -> PR {
    let imm = i.read_f64()?;
    let imm_var = ctxt.create_var(ValType::f64());
    let const_instr = Constant {
        imm: imm.into(),
        out1: imm_var.id,
        out1_type: NumType::F64,
    };
    o.write(const_instr);
    ctxt.push_var(imm_var);
    Ok(())
}
