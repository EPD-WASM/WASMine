use super::*;
use wasm_types::*;

#[derive(Debug, Clone)]
pub struct IUnaryInstruction {
    pub types: NumType,
    pub op: IUnaryOp,
    pub in1: VariableID,
    pub out1: VariableID,
}

impl Instruction for IUnaryInstruction {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.write_instruction_type(InstructionType::Numeric(
            NumericInstructionCategory::IUnary(self.op.clone()),
        ));
        o.write_value_type(ValType::Number(self.types));
        o.write_variable(self.in1);
        o.write_variable(self.out1);
    }

    fn deserialize(
        i: &mut InstructionDecoder,
        r#type: InstructionType,
    ) -> Result<Self, DecodingError> {
        let op = match r#type {
            InstructionType::Numeric(NumericInstructionCategory::IUnary(op)) => op,
            _ => return Err(DecodingError::TypeMismatch),
        };
        Ok(IUnaryInstruction {
            types: extract_numtype!(i.read_value_type()?),
            op,
            in1: i.read_variable()?,
            out1: i.read_variable()?,
        })
    }
}

impl Display for IUnaryInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "%{}: {} = {} {} %{}",
            self.out1, self.types, self.op, self.types, self.in1
        )
    }
}

#[derive(Debug, Clone)]
pub struct FUnaryInstruction {
    pub types: NumType,
    pub op: FUnaryOp,
    pub in1: VariableID,
    pub out1: VariableID,
}

impl Instruction for FUnaryInstruction {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.write_instruction_type(InstructionType::Numeric(
            NumericInstructionCategory::FUnary(self.op.clone()),
        ));
        o.write_value_type(ValType::Number(self.types));
        o.write_variable(self.in1);
        o.write_variable(self.out1);
    }

    fn deserialize(
        i: &mut InstructionDecoder,
        r#type: InstructionType,
    ) -> Result<Self, DecodingError> {
        let op = match r#type {
            InstructionType::Numeric(NumericInstructionCategory::FUnary(op)) => op,
            _ => return Err(DecodingError::TypeMismatch),
        };
        Ok(FUnaryInstruction {
            types: extract_numtype!(i.read_value_type()?),
            op,
            in1: i.read_variable()?,
            out1: i.read_variable()?,
        })
    }
}

impl Display for FUnaryInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "%{}: {} = {} {} %{}",
            self.out1, self.types, self.op, self.types, self.in1
        )
    }
}
