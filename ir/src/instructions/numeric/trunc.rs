use super::*;
use wasm_types::*;

#[derive(Debug, Clone)]
pub struct TruncSaturationInstruction {
    pub in1: VariableID,
    pub out1: VariableID,
    pub in1_type: NumType,
    pub out1_type: NumType,
    pub signed: bool,
}

impl Instruction for TruncSaturationInstruction {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.write_variable(self.in1);
        o.write_variable(self.out1);
        o.write_value_type(ValType::Number(self.in1_type));
        o.write_value_type(ValType::Number(self.out1_type));
        o.write_immediate(self.signed as u8);
    }

    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let in1 = i.read_variable()?;
        let out1 = i.read_variable()?;
        let in1_type = extract_numtype!(i.read_value_type()?);
        let out1_type = extract_numtype!(i.read_value_type()?);
        let signed = i.read_immediate::<u8>()? != 0;
        Ok(TruncSaturationInstruction {
            in1,
            out1,
            in1_type,
            out1_type,
            signed,
        })
    }
}
