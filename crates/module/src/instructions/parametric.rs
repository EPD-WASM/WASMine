use super::*;
use wasm_types::*;

#[derive(Debug, Clone)]
pub struct DropInstruction {}

impl Instruction for DropInstruction {
    fn deserialize(_: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        unimplemented!(
            "Drop instructions are not serialized and can therefore not be deserialized."
        )
    }
}

impl Display for DropInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "nop (orig: drop)")
    }
}

#[derive(Debug, Clone)]
pub struct SelectInstruction {
    pub input_vals: [VariableID; 2],
    pub select_val: VariableID,
    pub out1: VariableID,
}

impl Instruction for SelectInstruction {
    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let input_vals = [i.read_variable()?, i.read_variable()?];
        let select_val = i.read_variable()?;
        let out1 = i.read_variable()?;
        Ok(SelectInstruction {
            input_vals,
            select_val,
            out1,
        })
    }
}

impl Display for SelectInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "%{} = select %{} %{} %{}",
            self.out1, self.input_vals[0], self.input_vals[1], self.select_val
        )
    }
}
