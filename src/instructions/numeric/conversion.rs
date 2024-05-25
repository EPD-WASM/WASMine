use super::*;
use wasm_types::*;

#[derive(Debug, Clone)]
pub(crate) struct WrapInstruction {
    in1: VariableID,
    out1: VariableID,
}

impl Instruction for WrapInstruction {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.write_instruction_type(InstructionType::Numeric(
            NumericInstructionCategory::Conversion(ConversionOp::Wrap),
        ));
        o.write_variable(self.in1);
        o.write_variable(self.out1);
    }

    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        Ok(WrapInstruction {
            in1: i.read_variable()?,
            out1: i.read_variable()?,
        })
    }
}

impl Display for WrapInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "%{} = wrap %{}", self.out1, self.in1)
    }
}

pub(crate) fn i32_wrap_i64(
    ctxt: &mut Context,
    _: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let in_ = ctxt.pop_var_with_type(&ValType::Number(NumType::I64));
    let out = ctxt.create_var(ValType::Number(NumType::I32));
    o.write(WrapInstruction {
        in1: in_.id,
        out1: out.id,
    });
    ctxt.push_var(out);
    Ok(())
}

#[derive(Debug, Clone)]
pub(crate) struct TruncInstruction {
    in1: VariableID,
    in1_type: NumType,

    out1: VariableID,
    out1_type: NumType,

    signed: bool,
}

impl Instruction for TruncInstruction {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.write_instruction_type(InstructionType::Numeric(
            NumericInstructionCategory::Conversion(ConversionOp::Trunc),
        ));
        o.write_variable(self.in1);
        o.write_value_type(ValType::Number(self.in1_type));

        o.write_variable(self.out1);
        o.write_value_type(ValType::Number(self.out1_type));

        o.write_immediate(self.signed as u8);
    }

    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        Ok(TruncInstruction {
            in1: i.read_variable()?,
            in1_type: extract_numtype!(i.read_value_type()?),
            out1: i.read_variable()?,
            out1_type: extract_numtype!(i.read_value_type()?),
            signed: i.read_immediate::<u8>()? != 0,
        })
    }
}

impl Display for TruncInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "%{}: {} = trunc {} {} %{}",
            self.out1,
            self.out1_type,
            if self.signed { "signed" } else { "unsigned" },
            self.in1_type,
            self.in1
        )
    }
}

pub(crate) fn parse_trunc(
    ctxt: &mut Context,
    o: &mut InstructionEncoder,
    input_type: NumType,
    out_type: NumType,
    signed: bool,
) -> ParseResult {
    let in_ = ctxt.pop_var_with_type(&ValType::Number(input_type));
    let out = ctxt.create_var(ValType::Number(out_type));
    o.write(TruncInstruction {
        in1: in_.id,
        in1_type: input_type,
        out1: out.id,
        out1_type: out_type,
        signed,
    });
    ctxt.push_var(out);
    Ok(())
}

#[rustfmt::skip]
mod trunc_specializations {
    use super::*;

    pub(crate) fn i32_trunc_f32_s(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {parse_trunc(ctxt, o, NumType::F32, NumType::I32, true)}
    pub(crate) fn i32_trunc_f32_u(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {parse_trunc(ctxt, o, NumType::F32, NumType::I32, false)}
    pub(crate) fn i32_trunc_f64_s(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {parse_trunc(ctxt, o, NumType::F64, NumType::I32, true)}
    pub(crate) fn i32_trunc_f64_u(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {parse_trunc(ctxt, o, NumType::F64, NumType::I32, false)}

    pub(crate) fn i64_trunc_f32_s(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {parse_trunc(ctxt, o, NumType::F32, NumType::I64, true)}
    pub(crate) fn i64_trunc_f32_u(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {parse_trunc(ctxt, o, NumType::F32, NumType::I64, false)}
    pub(crate) fn i64_trunc_f64_s(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {parse_trunc(ctxt, o, NumType::F64, NumType::I64, true)}
    pub(crate) fn i64_trunc_f64_u(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {parse_trunc(ctxt, o, NumType::F64, NumType::I64, false)}
}
pub(crate) use trunc_specializations::*;

#[derive(Debug, Clone)]
pub(crate) struct ConvertInstruction {
    in1: VariableID,
    // TODO: This can be inferred from the variable id and is therefore redundant
    in1_type: NumType,

    out1: VariableID,
    out1_type: NumType,

    signed: bool,
}

impl Instruction for ConvertInstruction {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.write_instruction_type(InstructionType::Numeric(
            NumericInstructionCategory::Conversion(ConversionOp::Convert),
        ));
        o.write_variable(self.in1);
        o.write_value_type(ValType::Number(self.in1_type));

        o.write_variable(self.out1);
        o.write_value_type(ValType::Number(self.out1_type));

        o.write_immediate(self.signed as u8);
    }

    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        Ok(ConvertInstruction {
            in1: i.read_variable()?,
            in1_type: extract_numtype!(i.read_value_type()?),
            out1: i.read_variable()?,
            out1_type: extract_numtype!(i.read_value_type()?),
            signed: i.read_immediate::<u8>()? != 0,
        })
    }
}

impl Display for ConvertInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "%{}: {} = convert {} {} %{}",
            self.out1,
            self.out1_type,
            if self.signed { "signed" } else { "unsigned" },
            self.in1_type,
            self.in1
        )
    }
}

pub(crate) fn parse_convert(
    ctxt: &mut Context,
    o: &mut InstructionEncoder,
    input_type: NumType,
    out_type: NumType,
    signed: bool,
) -> ParseResult {
    let in_ = ctxt.pop_var_with_type(&ValType::Number(input_type));
    let out = ctxt.create_var(ValType::Number(out_type));
    o.write(ConvertInstruction {
        in1: in_.id,
        in1_type: input_type,
        out1: out.id,
        out1_type: out_type,
        signed,
    });
    ctxt.push_var(out);
    Ok(())
}

#[rustfmt::skip]
mod convert_specializations {
    use super::*;

    pub(crate) fn f32_convert_i32_s(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {parse_convert(ctxt, o, NumType::I32, NumType::F32, true)}
    pub(crate) fn f32_convert_i32_u(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {parse_convert(ctxt, o, NumType::I32, NumType::F32, false)}
    pub(crate) fn f32_convert_i64_s(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {parse_convert(ctxt, o, NumType::I64, NumType::F32, true)}
    pub(crate) fn f32_convert_i64_u(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {parse_convert(ctxt, o, NumType::I64, NumType::F32, false)}

    pub(crate) fn f64_convert_i32_s(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {parse_convert(ctxt, o, NumType::I32, NumType::F64, true)}
    pub(crate) fn f64_convert_i32_u(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {parse_convert(ctxt, o, NumType::I32, NumType::F64, false)}
    pub(crate) fn f64_convert_i64_s(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {parse_convert(ctxt, o, NumType::I64, NumType::F64, true)}
    pub(crate) fn f64_convert_i64_u(ctxt: &mut C, _: &mut I, o: &mut O) -> PR {parse_convert(ctxt, o, NumType::I64, NumType::F64, false)}
}
pub(crate) use convert_specializations::*;

#[derive(Debug, Clone)]
pub(crate) struct ReinterpretInstruction {
    in1: VariableID,
    in1_type: NumType,

    out1: VariableID,
    out1_type: NumType,
}

impl Instruction for ReinterpretInstruction {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.write_instruction_type(InstructionType::Numeric(
            NumericInstructionCategory::Conversion(ConversionOp::Reinterpret),
        ));

        o.write_variable(self.in1);
        o.write_value_type(ValType::Number(self.in1_type));

        o.write_variable(self.out1);
        o.write_value_type(ValType::Number(self.out1_type));
    }

    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        Ok(ReinterpretInstruction {
            in1: i.read_variable()?,
            in1_type: extract_numtype!(i.read_value_type()?),
            out1: i.read_variable()?,
            out1_type: extract_numtype!(i.read_value_type()?),
        })
    }
}

impl Display for ReinterpretInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "%{}: {} = reinterpret {} %{}",
            self.out1, self.out1_type, self.in1_type, self.in1
        )
    }
}

pub(crate) fn parse_reinterpret(
    ctxt: &mut Context,
    _: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
    input_type: NumType,
    out_type: NumType,
) -> ParseResult {
    let in_ = ctxt.pop_var_with_type(&ValType::Number(input_type));
    let out = ctxt.create_var(ValType::Number(out_type));
    o.write(ReinterpretInstruction {
        in1: in_.id,
        in1_type: input_type,
        out1: out.id,
        out1_type: out_type,
    });
    ctxt.push_var(out);
    Ok(())
}

#[rustfmt::skip]
mod reinterpret_specializations {
    use super::*;

    pub(crate) fn i32_reinterpret_f32(ctxt: &mut C, i: &mut I, o: &mut O) -> PR {parse_reinterpret(ctxt, i, o, NumType::F32, NumType::I32)}
    pub(crate) fn i64_reinterpret_f64(ctxt: &mut C, i: &mut I, o: &mut O) -> PR {parse_reinterpret(ctxt, i, o, NumType::F64, NumType::I64)}
    pub(crate) fn f32_reinterpret_i32(ctxt: &mut C, i: &mut I, o: &mut O) -> PR {parse_reinterpret(ctxt, i, o, NumType::I32, NumType::F32)}
    pub(crate) fn f64_reinterpret_i64(ctxt: &mut C, i: &mut I, o: &mut O) -> PR {parse_reinterpret(ctxt, i, o, NumType::I64, NumType::F64)}
}
pub(crate) use reinterpret_specializations::*;

#[derive(Debug, Clone)]
pub(crate) struct ExtendInstruction {
    in1: VariableID,
    in1_type: NumType,

    input_size: u8,

    out1: VariableID,
    out1_type: NumType,
}

impl Instruction for ExtendInstruction {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.write_instruction_type(InstructionType::Numeric(
            NumericInstructionCategory::Conversion(ConversionOp::Extend),
        ));
        o.write_variable(self.in1);
        o.write_value_type(ValType::Number(self.in1_type));

        o.write_immediate(self.input_size);

        o.write_variable(self.out1);
        o.write_value_type(ValType::Number(self.out1_type));
    }

    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        Ok(ExtendInstruction {
            in1: i.read_variable()?,
            in1_type: extract_numtype!(i.read_value_type()?),
            input_size: i.read_immediate()?,
            out1: i.read_variable()?,
            out1_type: extract_numtype!(i.read_value_type()?),
        })
    }
}

impl Display for ExtendInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "%{}: {} = extend i{} %{}",
            self.out1, self.out1_type, self.input_size, self.in1
        )
    }
}

pub(crate) fn parse_extend(
    ctxt: &mut Context,
    _: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
    input_size: u8,
    out_type: NumType,
) -> ParseResult {
    let in_ = ctxt.pop_var_with_type(&ValType::Number(out_type));
    let out = ctxt.create_var(ValType::Number(out_type));
    o.write(ExtendInstruction {
        in1: in_.id,
        in1_type: out_type,
        input_size,
        out1: out.id,
        out1_type: out_type,
    });
    ctxt.push_var(out);
    Ok(())
}

#[rustfmt::skip]
mod extend_specializations {
    use super::*;

    pub(crate) fn i32_extend8_s(ctxt: &mut C, i: &mut I, o: &mut O) -> PR {parse_extend(ctxt, i, o, 8, NumType::I32)}
    pub(crate) fn i32_extend16_s(ctxt: &mut C, i: &mut I, o: &mut O) -> PR {parse_extend(ctxt, i, o, 16, NumType::I32)}
    pub(crate) fn i64_extend8_s(ctxt: &mut C, i: &mut I, o: &mut O) -> PR {parse_extend(ctxt, i, o, 8, NumType::I64)}
    pub(crate) fn i64_extend16_s(ctxt: &mut C, i: &mut I, o: &mut O) -> PR {parse_extend(ctxt, i, o, 16, NumType::I64)}
    pub(crate) fn i64_extend32_s(ctxt: &mut C, i: &mut I, o: &mut O) -> PR {parse_extend(ctxt, i, o, 32, NumType::I64)}
}
pub(crate) use extend_specializations::*;

#[derive(Debug, Clone)]
pub(crate) struct ExtendTypeInstruction {
    signed: bool,
    in1: VariableID,
    out1: VariableID,
}

impl Instruction for ExtendTypeInstruction {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.write_instruction_type(InstructionType::Numeric(
            NumericInstructionCategory::Conversion(ConversionOp::Extend),
        ));
        o.write_immediate(self.signed as u8);
        o.write_variable(self.in1);
        o.write_variable(self.out1);
    }

    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        Ok(ExtendTypeInstruction {
            signed: i.read_immediate::<u8>()? != 0,
            in1: i.read_variable()?,
            out1: i.read_variable()?,
        })
    }
}

impl Display for ExtendTypeInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "%{}: i64 = extend {}32 {}",
            self.out1,
            if self.signed { "s" } else { "u" },
            self.in1
        )
    }
}

pub(crate) fn parse_extend_type(
    ctxt: &mut Context,
    _: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
    signed: bool,
) -> ParseResult {
    let in_ = ctxt.pop_var_with_type(&ValType::Number(NumType::I32));
    let out = ctxt.create_var(ValType::Number(NumType::I64));
    o.write(ExtendTypeInstruction {
        signed,
        out1: out.id,
        in1: in_.id,
    });
    ctxt.push_var(out);
    Ok(())
}

#[rustfmt::skip]
mod extend_type_specializations {
    use super::*;

    pub(crate) fn i64_extend_i32_s(ctxt: &mut C, i: &mut I, o: &mut O) -> PR {parse_extend_type(ctxt, i, o, true)}
    pub(crate) fn i64_extend_i32_u(ctxt: &mut C, i: &mut I, o: &mut O) -> PR {parse_extend_type(ctxt, i, o, false)}
}
pub(crate) use extend_type_specializations::*;

#[derive(Debug, Clone)]
pub(crate) struct DemoteInstruction {
    in1: VariableID,
    out1: VariableID,
}

impl Instruction for DemoteInstruction {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.write_instruction_type(InstructionType::Numeric(
            NumericInstructionCategory::Conversion(ConversionOp::Demote),
        ));
        o.write_variable(self.in1);
        o.write_variable(self.out1);
    }

    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        Ok(DemoteInstruction {
            in1: i.read_variable()?,
            out1: i.read_variable()?,
        })
    }
}

impl Display for DemoteInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "%{}: f32 = demote f64 %{}", self.out1, self.in1)
    }
}

pub(crate) fn f32_demote_f64(
    ctxt: &mut Context,
    _: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let in_ = ctxt.pop_var_with_type(&ValType::Number(NumType::F64));
    let out = ctxt.create_var(ValType::Number(NumType::F32));
    o.write(DemoteInstruction {
        in1: in_.id,
        out1: out.id,
    });
    ctxt.push_var(out);
    Ok(())
}

#[derive(Debug, Clone)]
pub(crate) struct PromoteInstruction {
    in1: VariableID,
    out1: VariableID,
}

impl Instruction for PromoteInstruction {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.write_instruction_type(InstructionType::Numeric(
            NumericInstructionCategory::Conversion(ConversionOp::Promote),
        ));
        o.write_variable(self.in1);
        o.write_variable(self.out1);
    }

    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        Ok(PromoteInstruction {
            in1: i.read_variable()?,
            out1: i.read_variable()?,
        })
    }
}

impl Display for PromoteInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "%{}: f64 = promote f32 %{}", self.out1, self.in1)
    }
}

pub(crate) fn f64_promote_f32(
    ctxt: &mut Context,
    _: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let in_ = ctxt.pop_var_with_type(&ValType::Number(NumType::F32));
    let out = ctxt.create_var(ValType::Number(NumType::F64));
    o.write(PromoteInstruction {
        in1: in_.id,
        out1: out.id,
    });
    ctxt.push_var(out);
    Ok(())
}
