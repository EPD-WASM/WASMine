use super::*;
use crate::wasm_types::*;

#[derive(Debug, Clone)]
pub(crate) struct IRelationalInstruction {
    input_types: NumType,
    // the output type is always I32 for a bool result
    op: IRelationalOp,
    in1: VariableID,
    in2: VariableID,
    out1: VariableID,
}

impl Instruction for IRelationalInstruction {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.write_instruction_type(InstructionType::Numeric(
            NumericInstructionCategory::IRelational(self.op.clone()),
        ));
        o.write_value_type(ValType::Number(self.input_types));
        o.write_variable(self.in1);
        o.write_variable(self.in2);
        o.write_variable(self.out1);
    }

    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let op = match i.read_instruction_type()? {
            InstructionType::Numeric(NumericInstructionCategory::IRelational(op)) => op,
            _ => return Err(DecodingError::TypeMismatch),
        };
        Ok(IRelationalInstruction {
            input_types: extract_numtype!(i.read_value_type()?),
            op,
            in1: i.read_variable()?,
            in2: i.read_variable()?,
            out1: i.read_variable()?,
        })
    }
}

#[derive(Debug, Clone)]
pub(crate) struct FRelationalInstruction {
    input_types: NumType,
    // the output type is always I32 for a bool result
    op: FRelationalOp,
    in1: VariableID,
    in2: VariableID,
    out1: VariableID,
}

impl Instruction for FRelationalInstruction {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.write_instruction_type(InstructionType::Numeric(
            NumericInstructionCategory::FRelational(self.op.clone()),
        ));
        o.write_value_type(ValType::Number(self.input_types));
        o.write_variable(self.in1);
        o.write_variable(self.in2);
        o.write_variable(self.out1);
    }

    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let op = match i.read_instruction_type()? {
            InstructionType::Numeric(NumericInstructionCategory::FRelational(op)) => op,
            _ => return Err(DecodingError::TypeMismatch),
        };
        Ok(FRelationalInstruction {
            input_types: extract_numtype!(i.read_value_type()?),
            op,
            in1: i.read_variable()?,
            in2: i.read_variable()?,
            out1: i.read_variable()?,
        })
    }
}

fn i_arith(ctxt: &mut C, o: &mut O, op: IRelationalOp, type_: NumType) -> PR {
    let in1 = ctxt.pop_var_with_type(&ValType::Number(type_));
    let in2 = ctxt.pop_var_with_type(&ValType::Number(type_));
    let out = ctxt.create_var(ValType::Number(NumType::I32));
    o.write(IRelationalInstruction {
        input_types: type_,
        op,
        in1: in1.id,
        in2: in2.id,
        out1: out.id,
    });
    ctxt.push_var(out);
    Ok(())
}

fn f_arith(ctxt: &mut C, o: &mut O, op: FRelationalOp, type_: NumType) -> PR {
    let in1 = ctxt.pop_var_with_type(&ValType::Number(type_));
    let in2 = ctxt.pop_var_with_type(&ValType::Number(type_));
    let out = ctxt.create_var(ValType::Number(NumType::I32));
    o.write(FRelationalInstruction {
        input_types: type_,
        op,
        in1: in1.id,
        in2: in2.id,
        out1: out.id,
    });
    ctxt.push_var(out);
    Ok(())
}

#[rustfmt::skip]
mod specializations {
    use super::*;
    pub(crate) fn i32_eq(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {i_arith(ctxt, o, IRelationalOp::Eq, NumType::I32)}
    pub(crate) fn i32_ne(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {i_arith(ctxt, o, IRelationalOp::Ne, NumType::I32)}
    pub(crate) fn i32_lt_s(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {i_arith(ctxt, o, IRelationalOp::LtS, NumType::I32)}
    pub(crate) fn i32_lt_u(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {i_arith(ctxt, o, IRelationalOp::LtU, NumType::I32)}
    pub(crate) fn i32_gt_s(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {i_arith(ctxt, o, IRelationalOp::GtS, NumType::I32)}
    pub(crate) fn i32_gt_u(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {i_arith(ctxt, o, IRelationalOp::GtU, NumType::I32)}
    pub(crate) fn i32_le_s(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {i_arith(ctxt, o, IRelationalOp::LeS, NumType::I32)}
    pub(crate) fn i32_le_u(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {i_arith(ctxt, o, IRelationalOp::LeU, NumType::I32)}
    pub(crate) fn i32_ge_s(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {i_arith(ctxt, o, IRelationalOp::GeS, NumType::I32)}
    pub(crate) fn i32_ge_u(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {i_arith(ctxt, o, IRelationalOp::GeU, NumType::I32)}

    pub(crate) fn i64_eq(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {i_arith(ctxt, o, IRelationalOp::Eq, NumType::I64)}
    pub(crate) fn i64_ne(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {i_arith(ctxt, o, IRelationalOp::Ne, NumType::I64)}
    pub(crate) fn i64_lt_s(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {i_arith(ctxt, o, IRelationalOp::LtS, NumType::I64)}
    pub(crate) fn i64_lt_u(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {i_arith(ctxt, o, IRelationalOp::LtU, NumType::I64)}
    pub(crate) fn i64_gt_s(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {i_arith(ctxt, o, IRelationalOp::GtS, NumType::I64)}
    pub(crate) fn i64_gt_u(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {i_arith(ctxt, o, IRelationalOp::GtU, NumType::I64)}
    pub(crate) fn i64_le_s(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {i_arith(ctxt, o, IRelationalOp::LeS, NumType::I64)}
    pub(crate) fn i64_le_u(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {i_arith(ctxt, o, IRelationalOp::LeU, NumType::I64)}
    pub(crate) fn i64_ge_s(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {i_arith(ctxt, o, IRelationalOp::GeS, NumType::I64)}
    pub(crate) fn i64_ge_u(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {i_arith(ctxt, o, IRelationalOp::GeU, NumType::I64)}

    pub(crate) fn f32_eq(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {f_arith(ctxt, o, FRelationalOp::Eq, NumType::F32)}
    pub(crate) fn f32_ne(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {f_arith(ctxt, o, FRelationalOp::Ne, NumType::F32)}
    pub(crate) fn f32_lt(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {f_arith(ctxt, o, FRelationalOp::Lt, NumType::F32)}
    pub(crate) fn f32_gt(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {f_arith(ctxt, o, FRelationalOp::Gt, NumType::F32)}
    pub(crate) fn f32_le(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {f_arith(ctxt, o, FRelationalOp::Le, NumType::F32)}
    pub(crate) fn f32_ge(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {f_arith(ctxt, o, FRelationalOp::Ge, NumType::F32)}

    pub(crate) fn f64_eq(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {f_arith(ctxt, o, FRelationalOp::Eq, NumType::F64)}
    pub(crate) fn f64_ne(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {f_arith(ctxt, o, FRelationalOp::Ne, NumType::F64)}
    pub(crate) fn f64_lt(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {f_arith(ctxt, o, FRelationalOp::Lt, NumType::F64)}
    pub(crate) fn f64_gt(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {f_arith(ctxt, o, FRelationalOp::Gt, NumType::F64)}
    pub(crate) fn f64_le(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {f_arith(ctxt, o, FRelationalOp::Le, NumType::F64)}
    pub(crate) fn f64_ge(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {f_arith(ctxt, o, FRelationalOp::Ge, NumType::F64)}
}
pub(crate) use specializations::*;
