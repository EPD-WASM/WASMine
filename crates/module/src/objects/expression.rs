use super::value::{ConstantValue, Number, Value};
use crate::{
    basic_block::BasicBlockStorage,
    instructions::{Constant, GlobalGetInstruction, ReferenceFunctionInstruction},
    objects::value::Reference,
    DecodingError, InstructionDecoder, ModuleMetadata,
};
use thiserror::Error;
use wasm_types::{
    GlobalType, InstructionType, NumericInstructionCategory, ReferenceInstructionType, ValType,
    VariableInstructionType,
};

#[derive(Debug, Clone, Default)]
pub struct ConstantExpression {
    pub expression: BasicBlockStorage,
}

#[derive(Debug, Error)]
pub enum ConstantExpressionError {
    #[error("Constant expression error: {0}")]
    Msg(String),
    #[error("Error during decoding for constant expression: {0}")]
    DecodingError(#[from] DecodingError),
}

impl ConstantExpression {
    pub fn eval(self, m: &ModuleMetadata) -> Result<ConstantValue, ConstantExpressionError> {
        debug_assert_eq!(self.expression.instruction_storage.len(), 1);
        let mut decoder = InstructionDecoder::new(self.expression);
        let instr = decoder.read_instruction_type()?;
        // https://webassembly.github.io/spec/core/bikeshed/index.html#constant-expressions%E2%91%A0
        match instr {
            InstructionType::Numeric(NumericInstructionCategory::Constant) => {
                let constant_instruction = decoder.read::<Constant>(instr)?;
                let imm = constant_instruction.imm;
                let value = Value::from_raw(imm, ValType::Number(constant_instruction.out1_type));
                Ok(ConstantValue::V(value))
            }
            InstructionType::Variable(VariableInstructionType::GlobalGet) => {
                let instruction = decoder.read::<GlobalGetInstruction>(instr)?;
                let global_idx = instruction.global_idx;
                if global_idx > m.globals.len() as u32 {
                    return Err(ConstantExpressionError::Msg(format!(
                        "global index {global_idx} out of bounds",
                    )));
                }
                if !m.globals[global_idx as usize].import {
                    return Err(ConstantExpressionError::Msg(format!(
                        "constant global initializer expressions may only reference imported globals, found non-imported global index {global_idx}",

                    )));
                }
                if !matches!(m.globals[global_idx as usize].r#type, GlobalType::Const(_)) {
                    return Err(ConstantExpressionError::Msg(format!(
                        "global index {global_idx} is not a const global",
                    )));
                }
                Ok(ConstantValue::Global(global_idx))
            }
            InstructionType::Reference(ReferenceInstructionType::RefNull) => {
                Ok(ConstantValue::V(Value::Reference(Reference::Null)))
            }
            InstructionType::Reference(ReferenceInstructionType::RefFunc) => {
                let instruction = decoder.read::<ReferenceFunctionInstruction>(instr)?;
                if instruction.func_idx >= m.functions.len() as u32 {
                    return Err(ConstantExpressionError::Msg(format!(
                        "function index {} out of bounds",
                        instruction.func_idx
                    )));
                }
                Ok(ConstantValue::FuncPtr(instruction.func_idx))
            }
            _ => Err(ConstantExpressionError::Msg(format!(
                "invalid constant instruction `{instr:?}`",
            ))),
        }
    }
}

impl TryInto<u32> for Value {
    type Error = ConstantExpressionError;

    fn try_into(self) -> Result<u32, Self::Error> {
        match self {
            Value::Number(Number::I32(i)) | Value::Number(Number::U32(i)) => Ok(i),
            Value::Number(Number::I64(i)) | Value::Number(Number::U64(i)) => {
                i.try_into().map_err(|_| {
                    ConstantExpressionError::Msg(
                        "Invalid constant expression for index conversion".into(),
                    )
                })
            }
            Value::Number(Number::S32(i)) => i.try_into().map_err(|_| {
                ConstantExpressionError::Msg(
                    "Invalid constant expression for index conversion".into(),
                )
            }),
            Value::Number(Number::S64(i)) => i.try_into().map_err(|_| {
                ConstantExpressionError::Msg(
                    "Invalid constant expression for index conversion".into(),
                )
            }),
            Value::Number(Number::F32(f)) => {
                if f.trunc() == f && f >= 0.0 && f < u32::MAX as f32 {
                    Ok(f as u32)
                } else {
                    Err(ConstantExpressionError::Msg(
                        "Invalid constant expression for index conversion".into(),
                    ))
                }
            }
            Value::Number(Number::F64(f)) => {
                if f.trunc() == f && f >= 0.0 && f < u32::MAX as f64 {
                    Ok(f as u32)
                } else {
                    Err(ConstantExpressionError::Msg(
                        "Invalid constant expression for index conversion".into(),
                    ))
                }
            }
            _ => Err(ConstantExpressionError::Msg(
                "Invalid constant expression for index conversion".into(),
            )),
        }
    }
}

impl TryInto<Reference> for Value {
    type Error = ConstantExpressionError;

    fn try_into(self) -> Result<Reference, Self::Error> {
        match self {
            Value::Reference(r) => Ok(r),
            _ => Err(ConstantExpressionError::Msg(
                "Invalid constant expression for reference conversion".into(),
            )),
        }
    }
}
