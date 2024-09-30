use super::*;
use wasm_types::*;

#[derive(Debug, Clone)]
pub struct ITestInstruction {
    pub input_type: NumType,
    pub op: ITestOp,
    pub in1: VariableID,
    pub out1: VariableID,
}

impl Instruction for ITestInstruction {
    fn deserialize(
        i: &mut InstructionDecoder,
        r#type: InstructionType,
    ) -> Result<Self, DecodingError> {
        let op = match r#type {
            InstructionType::Numeric(NumericInstructionCategory::ITest(op)) => op,
            _ => return Err(DecodingError::TypeMismatch),
        };
        Ok(ITestInstruction {
            input_type: extract_numtype!(i.read_value_type()?),
            op,
            in1: i.read_variable()?,
            out1: i.read_variable()?,
        })
    }
}

impl Display for ITestInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "%{}: {} = {} {} %{}",
            self.out1, self.input_type, self.op, self.input_type, self.in1
        )
    }
}
