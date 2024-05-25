use super::*;
use wasm_types::*;

#[derive(Debug, Clone)]
pub(crate) struct IUnaryInstruction {
    types: NumType,
    op: IUnaryOp,
    in1: VariableID,
    out1: VariableID,
}

impl Instruction for IUnaryInstruction {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.write_instruction_type(InstructionType::Numeric(
            NumericInstructionCategory::IUnary(self.op.clone()),
        ));
        o.write_value_type(ValType::Number(self.types));
        o.write_variable(self.in1);
        o.write_variable(self.out1);
    }

    fn deserialize(
        i: &mut InstructionDecoder,
        r#type: InstructionType,
    ) -> Result<Self, DecodingError> {
        let op = match r#type {
            InstructionType::Numeric(NumericInstructionCategory::IUnary(op)) => op,
            _ => return Err(DecodingError::TypeMismatch),
        };
        Ok(IUnaryInstruction {
            types: extract_numtype!(i.read_value_type()?),
            op,
            in1: i.read_variable()?,
            out1: i.read_variable()?,
        })
    }
}

impl Display for IUnaryInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "%{}: {} = {} {} %{}",
            self.out1, self.types, self.op, self.types, self.in1
        )
    }
}

fn i_arith(ctxt: &mut C, o: &mut O, op: IUnaryOp, type_: NumType) -> PR {
    let in_ = ctxt.pop_var_with_type(&ValType::Number(type_));
    let out = ctxt.create_var(ValType::Number(type_));
    o.write(IUnaryInstruction {
        types: type_,
        op,
        in1: in_.id,
        out1: out.id,
    });
    ctxt.push_var(out);
    Ok(())
}

fn f_arith(ctxt: &mut C, o: &mut O, op: FUnaryOp, type_: NumType) -> PR {
    let in_ = ctxt.pop_var_with_type(&ValType::Number(type_));
    let out = ctxt.create_var(ValType::Number(type_));
    o.write(FUnaryInstruction {
        types: type_,
        op,
        in1: in_.id,
        out1: out.id,
    });
    ctxt.push_var(out);
    Ok(())
}

#[derive(Debug, Clone)]
pub(crate) struct FUnaryInstruction {
    types: NumType,
    op: FUnaryOp,
    in1: VariableID,
    out1: VariableID,
}

impl Instruction for FUnaryInstruction {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.write_instruction_type(InstructionType::Numeric(
            NumericInstructionCategory::FUnary(self.op.clone()),
        ));
        o.write_value_type(ValType::Number(self.types));
        o.write_variable(self.in1);
        o.write_variable(self.out1);
    }

    fn deserialize(
        i: &mut InstructionDecoder,
        r#type: InstructionType,
    ) -> Result<Self, DecodingError> {
        let op = match r#type {
            InstructionType::Numeric(NumericInstructionCategory::FUnary(op)) => op,
            _ => return Err(DecodingError::TypeMismatch),
        };
        Ok(FUnaryInstruction {
            types: extract_numtype!(i.read_value_type()?),
            op,
            in1: i.read_variable()?,
            out1: i.read_variable()?,
        })
    }
}

impl Display for FUnaryInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "%{}: {} = {} {} %{}",
            self.out1, self.types, self.op, self.types, self.in1
        )
    }
}

#[rustfmt::skip]
mod specializations {
    use super::*;
    pub(crate) fn i32_clz(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {i_arith(ctxt, o, IUnaryOp::Clz, NumType::I32)}
    pub(crate) fn i32_ctz(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {i_arith(ctxt, o, IUnaryOp::Ctz, NumType::I32)}
    pub(crate) fn i32_popcnt(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {i_arith(ctxt, o, IUnaryOp::Popcnt, NumType::I32)}

    pub(crate) fn i64_clz(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {i_arith(ctxt, o, IUnaryOp::Clz, NumType::I64)}
    pub(crate) fn i64_ctz(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {i_arith(ctxt, o, IUnaryOp::Ctz, NumType::I64)}
    pub(crate) fn i64_popcnt(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {i_arith(ctxt, o, IUnaryOp::Popcnt, NumType::I64)}

    pub(crate) fn f32_abs(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {f_arith(ctxt, o, FUnaryOp::Abs, NumType::F32)}
    pub(crate) fn f32_neg(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {f_arith(ctxt, o, FUnaryOp::Neg, NumType::F32)}
    pub(crate) fn f32_sqrt(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {f_arith(ctxt, o, FUnaryOp::Sqrt, NumType::F32)}
    pub(crate) fn f32_ceil(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {f_arith(ctxt, o, FUnaryOp::Ceil, NumType::F32)}
    pub(crate) fn f32_floor(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {f_arith(ctxt, o, FUnaryOp::Floor, NumType::F32)}
    pub(crate) fn f32_trunc(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {f_arith(ctxt, o, FUnaryOp::Trunc, NumType::F32)}
    pub(crate) fn f32_nearest(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {f_arith(ctxt, o, FUnaryOp::Nearest, NumType::F32)}

    pub(crate) fn f64_abs(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {f_arith(ctxt, o, FUnaryOp::Abs, NumType::F64)}
    pub(crate) fn f64_neg(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {f_arith(ctxt, o, FUnaryOp::Neg, NumType::F64)}
    pub(crate) fn f64_sqrt(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {f_arith(ctxt, o, FUnaryOp::Sqrt, NumType::F64)}
    pub(crate) fn f64_ceil(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {f_arith(ctxt, o, FUnaryOp::Ceil, NumType::F64)}
    pub(crate) fn f64_floor(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {f_arith(ctxt, o, FUnaryOp::Floor, NumType::F64)}
    pub(crate) fn f64_trunc(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {f_arith(ctxt, o, FUnaryOp::Trunc, NumType::F64)}
    pub(crate) fn f64_nearest(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {f_arith(ctxt, o, FUnaryOp::Nearest, NumType::F64)}
}
pub(crate) use specializations::*;
