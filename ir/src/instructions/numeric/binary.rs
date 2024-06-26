use super::*;
use wasm_types::*;

#[derive(Debug, Clone)]
pub struct IBinaryInstruction {
    pub types: NumType,
    pub op: IBinaryOp,
    pub lhs: VariableID,
    pub rhs: VariableID,
    pub out1: VariableID,
}

impl Instruction for IBinaryInstruction {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.write_instruction_type(InstructionType::Numeric(
            NumericInstructionCategory::IBinary(self.op.clone()),
        ));
        o.write_value_type(ValType::Number(self.types));
        o.write_variable(self.lhs);
        o.write_variable(self.rhs);
        o.write_variable(self.out1);
    }

    fn deserialize(i: &mut InstructionDecoder, t: InstructionType) -> Result<Self, DecodingError> {
        let op = match t {
            InstructionType::Numeric(NumericInstructionCategory::IBinary(op)) => op,
            _ => return Err(DecodingError::TypeMismatch),
        };
        Ok(IBinaryInstruction {
            types: extract_numtype!(i.read_value_type()?),
            op,
            lhs: i.read_variable()?,
            rhs: i.read_variable()?,
            out1: i.read_variable()?,
        })
    }
}

impl Display for IBinaryInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "%{}: {} = {} {} %{} %{}",
            self.out1, self.types, self.op, self.types, self.lhs, self.rhs
        )
    }
}

#[derive(Debug, Clone)]
pub struct FBinaryInstruction {
    pub types: NumType,
    pub op: FBinaryOp,
    pub lhs: VariableID,
    pub rhs: VariableID,
    pub out1: VariableID,
}

impl Instruction for FBinaryInstruction {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.write_instruction_type(InstructionType::Numeric(
            NumericInstructionCategory::FBinary(self.op.clone()),
        ));
        o.write_value_type(ValType::Number(self.types));
        o.write_variable(self.lhs);
        o.write_variable(self.rhs);
        o.write_variable(self.out1);
    }

    fn deserialize(i: &mut InstructionDecoder, t: InstructionType) -> Result<Self, DecodingError> {
        let op = match t {
            InstructionType::Numeric(NumericInstructionCategory::FBinary(op)) => op,
            _ => return Err(DecodingError::TypeMismatch),
        };
        Ok(FBinaryInstruction {
            types: extract_numtype!(i.read_value_type()?),
            op,
            lhs: i.read_variable()?,
            rhs: i.read_variable()?,
            out1: i.read_variable()?,
        })
    }
}

impl Display for FBinaryInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "%{}: {} = {} {} %{} %{}",
            self.out1, self.types, self.op, self.types, self.lhs, self.rhs
        )
    }
}
