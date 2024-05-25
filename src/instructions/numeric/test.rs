use super::*;
use wasm_types::*;

#[derive(Debug, Clone)]
pub(crate) struct ITestInstruction {
    input_type: NumType,
    op: ITestOp,
    in1: VariableID,
    out1: VariableID,
}

impl Instruction for ITestInstruction {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.write_instruction_type(InstructionType::Numeric(NumericInstructionCategory::ITest(
            self.op.clone(),
        )));
        o.write_value_type(ValType::Number(self.input_type));
        o.write_variable(self.in1);
        o.write_variable(self.out1);
    }

    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let op = match i.read_instruction_type()? {
            InstructionType::Numeric(NumericInstructionCategory::ITest(op)) => op,
            _ => return Err(DecodingError::TypeMismatch),
        };
        Ok(ITestInstruction {
            input_type: extract_numtype!(i.read_value_type()?),
            op,
            in1: i.read_variable()?,
            out1: i.read_variable()?,
        })
    }
}

fn i_arith(ctxt: &mut C, o: &mut O, op: ITestOp, type_: NumType) -> PR {
    let in_ = ctxt.pop_var_with_type(&ValType::Number(type_));
    let out = ctxt.create_var(ValType::Number(NumType::I32));
    o.write(ITestInstruction {
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
    pub(crate) fn i32_eqz(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {i_arith(ctxt, o, ITestOp::Eqz, NumType::I32)}
    pub(crate) fn i64_eqz(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {i_arith(ctxt, o, ITestOp::Eqz, NumType::I64)}
}
pub(crate) use specializations::*;
