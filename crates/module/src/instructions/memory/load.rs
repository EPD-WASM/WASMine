use super::*;

#[derive(Debug, Clone)]
pub struct LoadInstruction {
    pub memarg: MemArg,
    pub out1: VariableID,
    pub out1_type: NumType,
    pub addr: VariableID,
    pub operation: LoadOp,
}

impl Instruction for LoadInstruction {
    fn deserialize(i: &mut InstructionDecoder, t: InstructionType) -> Result<Self, DecodingError> {
        let align = i.read_immediate()?;
        let offset = i.read_immediate()?;
        let addr = i.read_variable()?;
        let out1 = i.read_variable()?;
        let out1_type = extract_numtype!(i.read_value_type()?);
        let operation = match t {
            InstructionType::Memory(MemoryInstructionCategory::Load(op)) => op,
            _ => return Err(DecodingError::TypeMismatch),
        };
        Ok(LoadInstruction {
            memarg: MemArg { align, offset },
            out1,
            out1_type,
            addr,
            operation,
        })
    }
}

impl Display for LoadInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "%{} = load {} %{} (align: {}, offset: {})",
            self.out1, self.out1_type, self.addr, self.memarg.align, self.memarg.offset
        )
    }
}
