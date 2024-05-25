use super::*;

#[derive(Debug, Clone)]
pub(crate) struct MemorySizeInstruction {
    out1: VariableID,
}

impl Instruction for MemorySizeInstruction {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.write_instruction_type(InstructionType::Memory(MemoryInstructionCategory::Memory(
            MemoryOp::Size,
        )));
        o.write_variable(self.out1)
    }

    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let out1 = i.read_variable()?;
        Ok(MemorySizeInstruction { out1 })
    }
}

impl Display for MemorySizeInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "%{}: i32 = memory.size", self.out1)
    }
}

pub(crate) fn memory_size(
    ctxt: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    if i.read_byte()? != 0 {
        return Err(ParserError::Msg(
            "memory size instruction invalid encoding".into(),
        ));
    }

    let out = ctxt.create_var(ValType::Number(NumType::I32));
    o.write(MemorySizeInstruction { out1: out.id });
    ctxt.push_var(out);
    Ok(())
}

#[derive(Debug, Clone)]
pub(crate) struct MemoryGrowInstruction {
    in1: VariableID,
    out1: VariableID,
}

impl Display for MemoryGrowInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "%{}: i32 = memory.grow i32 %{}", self.out1, self.in1)
    }
}

impl Instruction for MemoryGrowInstruction {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.write_instruction_type(InstructionType::Memory(MemoryInstructionCategory::Memory(
            MemoryOp::Grow,
        )));
        o.write_variable(self.in1);
        o.write_variable(self.out1);
    }

    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let in1 = i.read_variable()?;
        let out1 = i.read_variable()?;
        Ok(MemoryGrowInstruction { in1, out1 })
    }
}

pub(crate) fn memory_grow(
    ctxt: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    if i.read_byte()? != 0 {
        return Err(ParserError::Msg(
            "memory grow instruction invalid encoding".into(),
        ));
    }

    let size_in = ctxt.pop_var_with_type(&ValType::Number(NumType::I32));
    let out = ctxt.create_var(ValType::Number(NumType::I32));
    o.write(MemoryGrowInstruction {
        in1: size_in.id,
        out1: out.id,
    });
    ctxt.push_var(out);
    Ok(())
}

#[derive(Debug, Clone)]
pub(crate) struct MemoryCopyInstruction {
    n: VariableID,
    s: VariableID,
    d: VariableID,
}

impl Instruction for MemoryCopyInstruction {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.write_instruction_type(InstructionType::Memory(MemoryInstructionCategory::Memory(
            MemoryOp::Copy,
        )));
        o.write_variable(self.n);
        o.write_variable(self.s);
        o.write_variable(self.d);
    }

    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let n = i.read_variable()?;
        let s = i.read_variable()?;
        let d = i.read_variable()?;
        Ok(MemoryCopyInstruction { n, s, d })
    }
}

impl Display for MemoryCopyInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "memory.copy i32 %{} i32 %{} i32 %{}",
            self.n, self.s, self.d
        )
    }
}

pub(crate) fn memory_copy(
    ctxt: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    if i.read_byte()? != 0 || i.read_byte()? != 0 {
        return Err(ParserError::Msg(
            "memory copy instruction invalid encoding".into(),
        ));
    }

    let n = ctxt.pop_var_with_type(&ValType::Number(NumType::I32));
    let s = ctxt.pop_var_with_type(&ValType::Number(NumType::I32));
    let d = ctxt.pop_var_with_type(&ValType::Number(NumType::I32));
    o.write(MemoryCopyInstruction {
        n: n.id,
        s: s.id,
        d: d.id,
    });
    Ok(())
}

#[derive(Debug, Clone)]
pub(crate) struct MemoryFillInstruction {
    n: VariableID,
    val: VariableID,
    d: VariableID,
}

impl Display for MemoryFillInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "memory.fill i32 %{} i32 %{} i32 %{}",
            self.n, self.val, self.d
        )
    }
}

impl Instruction for MemoryFillInstruction {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.write_instruction_type(InstructionType::Memory(MemoryInstructionCategory::Memory(
            MemoryOp::Fill,
        )));
        o.write_variable(self.n);
        o.write_variable(self.val);
        o.write_variable(self.d);
    }

    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let n = i.read_variable()?;
        let val = i.read_variable()?;
        let d = i.read_variable()?;
        Ok(MemoryFillInstruction { n, val, d })
    }
}

pub(crate) fn memory_fill(
    ctxt: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    if i.read_byte()? != 0 {
        return Err(ParserError::Msg(
            "memory fill instruction invalid encoding".into(),
        ));
    }

    let n = ctxt.pop_var_with_type(&ValType::Number(NumType::I32));
    let val = ctxt.pop_var_with_type(&ValType::Number(NumType::I32));
    let d = ctxt.pop_var_with_type(&ValType::Number(NumType::I32));
    o.write(MemoryFillInstruction {
        n: n.id,
        val: val.id,
        d: d.id,
    });
    Ok(())
}

#[derive(Debug, Clone)]
pub(crate) struct MemoryInitInstruction {
    data_idx: DataIdx,
    n: VariableID,
    s: VariableID,
    d: VariableID,
}

impl Instruction for MemoryInitInstruction {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.write_instruction_type(InstructionType::Memory(MemoryInstructionCategory::Memory(
            MemoryOp::Init,
        )));
        o.write_immediate(self.data_idx);
        o.write_variable(self.n);
        o.write_variable(self.s);
        o.write_variable(self.d);
    }

    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let data_idx = i.read_immediate()?;
        let n = i.read_variable()?;
        let s = i.read_variable()?;
        let d = i.read_variable()?;
        Ok(MemoryInitInstruction { data_idx, n, s, d })
    }
}

impl Display for MemoryInitInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "memory.init(i32 {}) i32 %{} i32 %{} i32 %{}",
            self.data_idx, self.n, self.s, self.d
        )
    }
}

pub(crate) fn memory_init(
    ctxt: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let data_idx: u32 = DataIdx::parse(i)?;

    if let Some(data_count) = ctxt.module.datacount {
        if data_idx >= data_count {
            ctxt.poison(ValidationError::Msg(
                "memory init instruction data index out of bounds".into(),
            ))
        }
    } else {
        ctxt.poison(ValidationError::Msg(
            "memory init instruction without data section".into(),
        ))
    }

    if i.read_byte()? != 0 {
        return Err(ParserError::Msg(
            "memory init instruction invalid encoding".into(),
        ));
    }
    let n = ctxt.pop_var_with_type(&ValType::Number(NumType::I32));
    let s = ctxt.pop_var_with_type(&ValType::Number(NumType::I32));
    let d = ctxt.pop_var_with_type(&ValType::Number(NumType::I32));

    o.write(MemoryInitInstruction {
        data_idx,
        n: n.id,
        s: s.id,
        d: d.id,
    });
    Ok(())
}

#[derive(Debug, Clone)]
pub(crate) struct DataDropInstruction {
    data_idx: DataIdx,
}

impl Instruction for DataDropInstruction {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.write_instruction_type(InstructionType::Memory(MemoryInstructionCategory::Memory(
            MemoryOp::Drop,
        )));
        o.write_immediate(self.data_idx)
    }

    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let data_idx = i.read_immediate()?;
        Ok(DataDropInstruction { data_idx })
    }
}

impl Display for DataDropInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "memory.drop(i32 {})", self.data_idx)
    }
}

pub(crate) fn data_drop(
    ctxt: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let data_idx = DataIdx::parse(i)?;
    if let Some(data_count) = ctxt.module.datacount {
        if data_idx >= data_count {
            ctxt.poison(ValidationError::Msg(
                "memory init instruction data index out of bounds".into(),
            ))
        }
    } else {
        ctxt.poison(ValidationError::Msg(
            "memory init instruction without data section".into(),
        ))
    }

    o.write(DataDropInstruction { data_idx });
    Ok(())
}
