use super::*;
use wasm_types::*;

#[derive(Debug, Clone)]
pub struct WrapInstruction {
    pub in1: VariableID,
    pub out1: VariableID,
}

impl Instruction for WrapInstruction {
    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        Ok(WrapInstruction {
            in1: i.read_variable()?,
            out1: i.read_variable()?,
        })
    }
}

impl Display for WrapInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "%{} = wrap %{}", self.out1, self.in1)
    }
}

#[derive(Debug, Clone)]
pub struct ConvertInstruction {
    pub in1: VariableID,
    // TODO: This can be inferred from the variable id and is therefore redundant
    pub in1_type: NumType,

    pub out1: VariableID,
    pub out1_type: NumType,

    pub signed: bool,
}

impl Instruction for ConvertInstruction {
    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        Ok(ConvertInstruction {
            in1: i.read_variable()?,
            in1_type: extract_numtype!(i.read_value_type()?),
            out1: i.read_variable()?,
            out1_type: extract_numtype!(i.read_value_type()?),
            signed: i.read_immediate::<u8>()? != 0,
        })
    }
}

impl Display for ConvertInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "%{}: {} = convert {} {} %{}",
            self.out1,
            self.out1_type,
            if self.signed { "signed" } else { "unsigned" },
            self.in1_type,
            self.in1
        )
    }
}

#[derive(Debug, Clone)]
pub struct ReinterpretInstruction {
    pub in1: VariableID,
    pub in1_type: NumType,

    pub out1: VariableID,
    pub out1_type: NumType,
}

impl Instruction for ReinterpretInstruction {
    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        Ok(ReinterpretInstruction {
            in1: i.read_variable()?,
            in1_type: extract_numtype!(i.read_value_type()?),
            out1: i.read_variable()?,
            out1_type: extract_numtype!(i.read_value_type()?),
        })
    }
}

impl Display for ReinterpretInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "%{}: {} = reinterpret {} %{}",
            self.out1, self.out1_type, self.in1_type, self.in1
        )
    }
}

#[derive(Debug, Clone)]
pub struct ExtendBitsInstruction {
    pub in1: VariableID,
    pub in1_type: NumType,

    pub input_size: u8,

    pub out1: VariableID,
    pub out1_type: NumType,
}

impl Instruction for ExtendBitsInstruction {
    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        Ok(ExtendBitsInstruction {
            in1: i.read_variable()?,
            in1_type: extract_numtype!(i.read_value_type()?),
            input_size: i.read_immediate()?,
            out1: i.read_variable()?,
            out1_type: extract_numtype!(i.read_value_type()?),
        })
    }
}

impl Display for ExtendBitsInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "%{}: {} = extend i{} %{}",
            self.out1, self.out1_type, self.input_size, self.in1
        )
    }
}

#[derive(Debug, Clone)]
pub struct ExtendTypeInstruction {
    pub signed: bool,
    pub in1: VariableID,
    pub out1: VariableID,
}

impl Instruction for ExtendTypeInstruction {
    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        Ok(ExtendTypeInstruction {
            signed: i.read_immediate::<u8>()? != 0,
            in1: i.read_variable()?,
            out1: i.read_variable()?,
        })
    }
}

impl Display for ExtendTypeInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "%{}: i64 = extend {}32 {}",
            self.out1,
            if self.signed { "s" } else { "u" },
            self.in1
        )
    }
}

#[derive(Debug, Clone)]
pub struct DemoteInstruction {
    pub in1: VariableID,
    pub out1: VariableID,
}

impl Instruction for DemoteInstruction {
    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        Ok(DemoteInstruction {
            in1: i.read_variable()?,
            out1: i.read_variable()?,
        })
    }
}

impl Display for DemoteInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "%{}: f32 = demote f64 %{}", self.out1, self.in1)
    }
}

#[derive(Debug, Clone)]
pub struct PromoteInstruction {
    pub in1: VariableID,
    pub out1: VariableID,
}

impl Instruction for PromoteInstruction {
    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        Ok(PromoteInstruction {
            in1: i.read_variable()?,
            out1: i.read_variable()?,
        })
    }
}

impl Display for PromoteInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "%{}: f64 = promote f32 %{}", self.out1, self.in1)
    }
}
