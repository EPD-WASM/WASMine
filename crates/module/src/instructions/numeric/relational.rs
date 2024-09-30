use super::*;
use wasm_types::*;

#[derive(Debug, Clone)]
pub struct IRelationalInstruction {
    pub input_types: NumType,
    // the output type is always I32 for a bool result
    pub op: IRelationalOp,
    pub in1: VariableID,
    pub in2: VariableID,
    pub out1: VariableID,
}

impl Instruction for IRelationalInstruction {
    fn deserialize(
        i: &mut InstructionDecoder,
        type_: InstructionType,
    ) -> Result<Self, DecodingError> {
        let op = match type_ {
            InstructionType::Numeric(NumericInstructionCategory::IRelational(op)) => op,
            _ => return Err(DecodingError::TypeMismatch),
        };
        Ok(IRelationalInstruction {
            input_types: extract_numtype!(i.read_value_type()?),
            op,
            in1: i.read_variable()?,
            in2: i.read_variable()?,
            out1: i.read_variable()?,
        })
    }
}

impl Display for IRelationalInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "%{}: {} = {} {} %{} %{}",
            self.out1, self.input_types, self.op, self.input_types, self.in1, self.in2
        )
    }
}

#[derive(Debug, Clone)]
pub struct FRelationalInstruction {
    pub input_types: NumType,
    // the output type is always I32 for a bool result
    pub op: FRelationalOp,
    pub in1: VariableID,
    pub in2: VariableID,
    pub out1: VariableID,
}

impl Instruction for FRelationalInstruction {
    fn deserialize(
        i: &mut InstructionDecoder,
        r#type: InstructionType,
    ) -> Result<Self, DecodingError> {
        let op = match r#type {
            InstructionType::Numeric(NumericInstructionCategory::FRelational(op)) => op,
            _ => return Err(DecodingError::TypeMismatch),
        };
        Ok(FRelationalInstruction {
            input_types: extract_numtype!(i.read_value_type()?),
            op,
            in1: i.read_variable()?,
            in2: i.read_variable()?,
            out1: i.read_variable()?,
        })
    }
}

impl Display for FRelationalInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "%{}: {} = {} {} %{} %{}",
            self.out1, self.input_types, self.op, self.input_types, self.in1, self.in2
        )
    }
}
