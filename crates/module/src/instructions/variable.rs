use super::*;
use wasm_types::*;

#[derive(Debug, Clone)]
pub struct LocalGetInstruction {
    pub local_idx: LocalIdx,
    pub out1: VariableID,
}

impl Instruction for LocalGetInstruction {
    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let local_idx = i.read_immediate()?;
        let out1 = i.read_variable()?;
        Ok(LocalGetInstruction { local_idx, out1 })
    }
}

impl Display for LocalGetInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "%{} = local.get(i32 {})", self.out1, self.local_idx)
    }
}

#[derive(Debug, Clone)]
pub struct GlobalGetInstruction {
    pub global_idx: GlobalIdx,
    pub out1: VariableID,
}

impl Instruction for GlobalGetInstruction {
    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let global_idx = i.read_immediate()?;
        let out1 = i.read_variable()?;
        Ok(GlobalGetInstruction { global_idx, out1 })
    }
}

impl Display for GlobalGetInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "%{} = global.get(i32 {})", self.out1, self.global_idx)
    }
}

#[derive(Debug, Clone)]
pub struct LocalSetInstruction {
    pub local_idx: LocalIdx,
    pub in1: VariableID,
}

impl Instruction for LocalSetInstruction {
    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let local_idx = i.read_immediate()?;
        let in1 = i.read_variable()?;
        Ok(LocalSetInstruction { local_idx, in1 })
    }
}

impl Display for LocalSetInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "local.set(i32 {}) %{}", self.local_idx, self.in1)
    }
}

#[derive(Debug, Clone)]
pub struct GlobalSetInstruction {
    pub global_idx: GlobalIdx,
    pub in1: VariableID,
}

impl Instruction for GlobalSetInstruction {
    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let global_idx = i.read_immediate()?;
        let in1 = i.read_variable()?;
        Ok(GlobalSetInstruction { global_idx, in1 })
    }
}

impl Display for GlobalSetInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "global.set(i32 {}) %{}", self.global_idx, self.in1)
    }
}

#[derive(Debug, Clone)]
pub struct LocalTeeInstruction {
    pub local_idx: LocalIdx,
    pub in1: VariableID,
    pub out1: VariableID,
}

impl Instruction for LocalTeeInstruction {
    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let local_idx = i.read_immediate()?;
        let in1 = i.read_variable()?;
        let out1 = i.read_variable()?;
        Ok(LocalTeeInstruction {
            local_idx,
            in1,
            out1,
        })
    }
}

impl Display for LocalTeeInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "%{} = local.tee(i32 {}) %{}",
            self.out1, self.local_idx, self.in1
        )
    }
}
