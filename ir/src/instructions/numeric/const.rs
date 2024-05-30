use super::*;
use wasm_types::*;

#[derive(Debug, Clone)]
pub struct Constant {
    pub imm: u64,
    pub out1: VariableID,
    pub out1_type: NumType,
}

impl Instruction for Constant {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.write_instruction_type(InstructionType::Numeric(
            NumericInstructionCategory::Constant,
        ));
        o.write_immediate(self.imm);
        o.write_variable(self.out1);
        o.write_value_type(ValType::Number(self.out1_type));
    }

    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let imm = i.read_immediate()?;
        let out1 = i.read_variable()?;
        let num_type = extract_numtype!(i.read_value_type().unwrap());
        Ok(Constant {
            imm,
            out1,
            out1_type: num_type,
        })
    }
}

impl Display for Constant {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "%{}: {} = const {}", self.out1, self.out1_type, self.imm)
    }
}
