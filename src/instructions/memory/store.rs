use super::*;

#[derive(Debug, Clone)]
pub(crate) struct StoreInstruction {
    memarg: MemArg,
    addr_in: VariableID,
    value_in: VariableID,
    operation: StoreOp,
}

impl Instruction for StoreInstruction {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.write_instruction_type(InstructionType::Memory(MemoryInstructionCategory::Store(
            self.operation.clone(),
        )));
        o.write_immediate(self.memarg.align);
        o.write_immediate(self.memarg.offset);
        o.write_variable(self.addr_in);
        o.write_variable(self.value_in);
    }

    fn deserialize(i: &mut InstructionDecoder, t: InstructionType) -> Result<Self, DecodingError> {
        let align = i.read_immediate()?;
        let offset = i.read_immediate()?;
        let addr_in = i.read_variable()?;
        let value_in = i.read_variable()?;
        let operation = match t {
            InstructionType::Memory(MemoryInstructionCategory::Store(op)) => op,
            _ => return Err(DecodingError::TypeMismatch),
        };
        Ok(StoreInstruction {
            memarg: MemArg { align, offset },
            addr_in,
            value_in,
            operation,
        })
    }
}

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

impl Display for StoreInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "store %{} %{} (align: {}, offset: {})",
            self.addr_in, self.value_in, self.memarg.align, self.memarg.offset
        )
    }
}
