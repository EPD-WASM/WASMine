use super::*;

#[derive(Debug, Clone)]
pub struct ReferenceIsNullInstruction {
    pub in1: VariableID,
    pub in1_type: ValType,
    pub out1: VariableID,
}

impl Instruction for ReferenceIsNullInstruction {
    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let in1 = i.read_variable()?;
        let in1_type = i.read_value_type()?;
        let out1 = i.read_variable()?;
        Ok(ReferenceIsNullInstruction {
            in1,
            in1_type,
            out1,
        })
    }
}

impl Display for ReferenceIsNullInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "%{} = ref.is_null {} %{}",
            self.out1, self.in1_type, self.in1
        )
    }
}

#[derive(Debug, Clone)]
pub struct ReferenceNullInstruction {
    pub out1: VariableID,
    pub out1_type: RefType,
}

impl Instruction for ReferenceNullInstruction {
    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let out1 = i.read_variable()?;
        let out1_type = match i.read_value_type()? {
            ValType::Reference(rt) => rt,
            _ => return Err(DecodingError::TypeMismatch),
        };
        Ok(ReferenceNullInstruction { out1, out1_type })
    }
}

impl Display for ReferenceNullInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "%{}: {} = ref.null", self.out1, self.out1_type)
    }
}

#[derive(Debug, Clone)]
pub struct ReferenceFunctionInstruction {
    pub out1: VariableID,
    pub func_idx: FuncIdx,
}

impl Instruction for ReferenceFunctionInstruction {
    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let out1 = i.read_variable()?;
        let func_idx = i.read_immediate()?;
        Ok(ReferenceFunctionInstruction { out1, func_idx })
    }
}

impl Display for ReferenceFunctionInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "%{} = ref.func {}", self.out1, self.func_idx)
    }
}
