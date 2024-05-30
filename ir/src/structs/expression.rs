use super::{
    module::Module,
    value::{Number, Value},
};
use crate::{
    basic_block::BasicBlockStorage,
    instructions::{Constant, GlobalGetInstruction, ReferenceFunctionInstruction},
    structs::value::Reference,
    DecodingError, InstructionDecoder,
};
use thiserror::Error;
use wasm_types::{
    InstructionType, NumType, NumericInstructionCategory, ReferenceInstructionType,
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
    pub fn eval(self, m: &Module) -> Result<Value, ConstantExpressionError> {
        debug_assert_eq!(self.expression.instruction_storage.len(), 1);
        let mut decoder = InstructionDecoder::new(self.expression);
        let instr = decoder.read_instruction_type()?;
        // https://webassembly.github.io/spec/core/bikeshed/index.html#constant-expressions%E2%91%A0
        match instr {
            InstructionType::Numeric(NumericInstructionCategory::Constant) => {
                let constant_instruction = decoder.read::<Constant>(instr)?;
                let imm = constant_instruction.imm;
                let value = match constant_instruction.out1_type {
                    NumType::I32 => Value::Number(Number::I32(imm.try_into().map_err(|_| {
                        ConstantExpressionError::Msg(format!(
                            "immediate {} out of bounds of global type i32",
                            imm
                        ))
                    })?)),
                    NumType::I64 => Value::Number(Number::I64(imm)),
                    NumType::F32 => Value::Number(Number::F32(f32::from_bits(imm as u32))),
                    NumType::F64 => Value::Number(Number::F64(f64::from_bits(imm))),
                };
                Ok(value)
            }
            InstructionType::Variable(VariableInstructionType::GlobalGet) => {
                let instruction = decoder.read::<GlobalGetInstruction>(instr)?;
                let global_idx = instruction.global_idx;
                if global_idx > m.globals.len() as u32 {
                    return Err(ConstantExpressionError::Msg(format!(
                        "global index {} out of bounds",
                        global_idx
                    )));
                }
                if !m.globals[global_idx as usize].import {
                    return Err(ConstantExpressionError::Msg(format!(
                        "constant global initializer expressions may only reference imported globals, found non-imported global index {}",
                        global_idx
                    )));
                }
                Ok(m.globals[global_idx as usize].init.clone())
            }
            InstructionType::Reference(ReferenceInstructionType::RefNull) => {
                Ok(Value::Reference(Reference::Null))
            }
            InstructionType::Reference(ReferenceInstructionType::RefFunc) => {
                let instruction = decoder.read::<ReferenceFunctionInstruction>(instr)?;
                if instruction.func_idx >= m.ir.functions.len() as u32 {
                    return Err(ConstantExpressionError::Msg(format!(
                        "function index {} out of bounds",
                        instruction.func_idx
                    )));
                }
                Ok(Value::Reference(Reference::Function(instruction.func_idx)))
            }
            _ => Err(ConstantExpressionError::Msg(format!(
                "invalid constant instruction `{:?}`",
                instr
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
