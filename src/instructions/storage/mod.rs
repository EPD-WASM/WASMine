mod decoder;
mod encoder;

pub(crate) use decoder::{DecodingError, InstructionDecoder};
pub(crate) use encoder::InstructionEncoder;

use super::VariableID;
use crate::instructions::*;
use crate::structs::instruction::ControlInstruction;
use std::{
    collections::VecDeque,
    fmt::{Display, Formatter},
};
use wasm_types::MemoryOp;

#[derive(Debug, Default, Clone)]
pub(crate) struct InstructionStorage {
    pub(crate) immediate_storage: VecDeque<u8>,
    pub(crate) variable_storage: VecDeque<VariableID>,
    pub(crate) type_storage: VecDeque<ValType>,
    pub(crate) instruction_storage: VecDeque<InstructionType>,
    pub(crate) terminator: ControlInstruction,
}

impl Display for InstructionStorage {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut decoder = InstructionDecoder::new(self.clone());
        while let Ok(i_type) = decoder.read_instruction_type() {
            #[rustfmt::skip]
            match i_type {
                InstructionType::Control(ControlInstructionType::Unreachable) => write!(f, "{:?}, ", decoder.read::<Unreachable>(i_type).unwrap()),
                InstructionType::Control(ControlInstructionType::Nop) => write!(f, "Nop, ",),
                InstructionType::Control(ControlInstructionType::Block) => write!(f, "{:?}, ", decoder.read::<Block>(i_type).unwrap()),
                InstructionType::Control(ControlInstructionType::Loop) => write!(f, "{:?}, ", decoder.read::<Loop>(i_type).unwrap()),
                InstructionType::Control(ControlInstructionType::IfElse) => write!(f, "{:?}, ", decoder.read::<IfElse>(i_type).unwrap()),
                InstructionType::Control(ControlInstructionType::Else) => write!(f, "{:?}, ", decoder.read::<Else>(i_type).unwrap()),
                InstructionType::Control(ControlInstructionType::End) => write!(f, "{:?}, ", decoder.read::<End>(i_type).unwrap()),
                InstructionType::Control(ControlInstructionType::Br) => write!(f, "{:?}, ", decoder.read::<Br>(i_type).unwrap()),
                InstructionType::Control(ControlInstructionType::BrIf) => write!(f, "{:?}, ", decoder.read::<BrIf>(i_type).unwrap()),
                InstructionType::Control(ControlInstructionType::BrTable) => write!(f, "{:?}, ", decoder.read::<BrTable>(i_type).unwrap()),
                InstructionType::Control(ControlInstructionType::Return) => write!(f, "{:?}, ", decoder.read::<Return>(i_type).unwrap()),
                InstructionType::Control(ControlInstructionType::Call) => write!(f, "{:?}, ", decoder.read::<Call>(i_type).unwrap()),
                InstructionType::Control(ControlInstructionType::CallIndirect) => write!(f, "{:?}, ", decoder.read::<CallIndirect>(i_type).unwrap()),

                InstructionType::Numeric(NumericInstructionCategory::Constant) => write!(f, "{:?}, ", decoder.read::<Constant>(i_type).unwrap()),
                InstructionType::Numeric(NumericInstructionCategory::IUnary(_)) => write!(f, "{:?}, ", decoder.read::<IUnaryInstruction>(i_type).unwrap()),
                InstructionType::Numeric(NumericInstructionCategory::FUnary(_)) => write!(f, "{:?}, ", decoder.read::<FUnaryInstruction>(i_type).unwrap()),
                InstructionType::Numeric(NumericInstructionCategory::IBinary(_)) => write!(f, "{:?}, ", decoder.read::<IBinaryInstruction>(i_type).unwrap()),
                InstructionType::Numeric(NumericInstructionCategory::FBinary(_)) => write!(f, "{:?}, ", decoder.read::<FBinaryInstruction>(i_type).unwrap()),
                InstructionType::Numeric(NumericInstructionCategory::IRelational(_)) => write!(f, "{:?}, ", decoder.read::<IRelationalInstruction>(i_type).unwrap()),
                InstructionType::Numeric(NumericInstructionCategory::FRelational(_)) => write!(f, "{:?}, ", decoder.read::<FRelationalInstruction>(i_type).unwrap()),
                InstructionType::Numeric(NumericInstructionCategory::Conversion(_)) => write!(f, "{:?}, ", decoder.read::<ConvertInstruction>(i_type).unwrap()),
                InstructionType::Numeric(NumericInstructionCategory::ITest(_)) => write!(f, "{:?}, ", decoder.read::<ITestInstruction>(i_type).unwrap()),

                InstructionType::Parametric(ParametricInstructionType::Drop) => write!(f, "{:?}, ", decoder.read::<DropInstruction>(i_type).unwrap()),
                InstructionType::Parametric(ParametricInstructionType::Select) => write!(f, "{:?}, ", decoder.read::<SelectInstruction>(i_type).unwrap()),

                InstructionType::Variable(VariableInstructionType::LocalGet) => write!(f, "{:?}, ", decoder.read::<LocalGetInstruction>(i_type).unwrap()),
                InstructionType::Variable(VariableInstructionType::LocalSet) => write!(f, "{:?}, ", decoder.read::<LocalSetInstruction>(i_type).unwrap()),
                InstructionType::Variable(VariableInstructionType::LocalTee) => write!(f, "{:?}, ", decoder.read::<TeeLocalInstruction>(i_type).unwrap()),
                InstructionType::Variable(VariableInstructionType::GlobalGet) => write!(f, "{:?}, ", decoder.read::<GlobalGetInstruction>(i_type).unwrap()),
                InstructionType::Variable(VariableInstructionType::GlobalSet) => write!(f, "{:?}, ", decoder.read::<GlobalSetInstruction>(i_type).unwrap()),

                InstructionType::Memory(MemoryInstructionCategory::Load(_)) => write!(f, "{:?}, ", decoder.read::<LoadInstruction>(i_type).unwrap()),
                InstructionType::Memory(MemoryInstructionCategory::Store(_)) => write!(f, "{:?}, ", decoder.read::<StoreInstruction>(i_type).unwrap()),
                InstructionType::Memory(MemoryInstructionCategory::Memory(MemoryOp::Grow)) => write!(f, "{:?}, ", decoder.read::<MemoryGrowInstruction>(i_type).unwrap()),
                InstructionType::Memory(MemoryInstructionCategory::Memory(MemoryOp::Size)) => write!(f, "{:?}, ", decoder.read::<MemorySizeInstruction>(i_type).unwrap()),

                _ => todo!("{:?}", i_type),
            }?;
        }
        Ok(())
    }
}
