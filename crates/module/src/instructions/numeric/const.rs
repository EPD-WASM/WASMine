use super::*;
use crate::objects::value::ValueRaw;
use wasm_types::*;

#[derive(Debug, Clone)]
pub struct Constant {
    pub imm: ValueRaw,
    pub out1: VariableID,
    pub out1_type: NumType,
}

impl Instruction for Constant {
    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let imm = i.read_immediate::<u64>()?.into();
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
        write!(
            f,
            "%{}: {} = const {:?}",
            self.out1, self.out1_type, self.imm
        )
    }
}
