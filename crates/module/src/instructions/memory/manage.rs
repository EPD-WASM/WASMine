use super::*;

#[derive(Debug, Clone)]
pub struct MemorySizeInstruction {
    pub out1: VariableID,
}

impl Instruction for MemorySizeInstruction {
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

#[derive(Debug, Clone)]
pub struct MemoryGrowInstruction {
    pub in1: VariableID,
    pub out1: VariableID,
}

impl Display for MemoryGrowInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "%{}: i32 = memory.grow i32 %{}", self.out1, self.in1)
    }
}

impl Instruction for MemoryGrowInstruction {
    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let in1 = i.read_variable()?;
        let out1 = i.read_variable()?;
        Ok(MemoryGrowInstruction { in1, out1 })
    }
}

#[derive(Debug, Clone)]
pub struct MemoryCopyInstruction {
    pub n: VariableID,
    pub s: VariableID,
    pub d: VariableID,
}

impl Instruction for MemoryCopyInstruction {
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

#[derive(Debug, Clone)]
pub struct MemoryFillInstruction {
    pub n: VariableID,
    pub val: VariableID,
    pub d: VariableID,
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
    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let n = i.read_variable()?;
        let val = i.read_variable()?;
        let d = i.read_variable()?;
        Ok(MemoryFillInstruction { n, val, d })
    }
}

#[derive(Debug, Clone)]
pub struct MemoryInitInstruction {
    pub data_idx: DataIdx,
    pub n: VariableID,
    pub s: VariableID,
    pub d: VariableID,
}

impl Instruction for MemoryInitInstruction {
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

#[derive(Debug, Clone)]
pub struct DataDropInstruction {
    pub data_idx: DataIdx,
}

impl Instruction for DataDropInstruction {
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
