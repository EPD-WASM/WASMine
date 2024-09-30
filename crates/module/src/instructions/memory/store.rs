use super::*;

#[derive(Debug, Clone)]
pub struct StoreInstruction {
    pub memarg: MemArg,
    pub addr_in: VariableID,
    pub value_in: VariableID,
    pub in_type: NumType,
    pub operation: StoreOp,
}

impl Instruction for StoreInstruction {
    fn deserialize(i: &mut InstructionDecoder, t: InstructionType) -> Result<Self, DecodingError> {
        let align = i.read_immediate()?;
        let offset = i.read_immediate()?;
        let addr_in = i.read_variable()?;
        let value_in = i.read_variable()?;
        let in_type = extract_numtype!(i.read_value_type()?);
        let operation = match t {
            InstructionType::Memory(MemoryInstructionCategory::Store(op)) => op,
            _ => return Err(DecodingError::TypeMismatch),
        };
        Ok(StoreInstruction {
            memarg: MemArg { align, offset },
            addr_in,
            value_in,
            operation,
            in_type,
        })
    }
}

impl Display for StoreInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "store %{} %{} (align: {}, offset: {})",
            self.addr_in, self.value_in, self.memarg.align, self.memarg.offset
        )
    }
}
