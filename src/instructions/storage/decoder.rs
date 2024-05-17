use super::InstructionStorage;
use crate::{
    instructions::*, structs::instruction::ControlInstruction, util::integer_traits::Integer,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum DecodingError {
    #[error("Decoding error: {0}")]
    DecodingError(String),
    #[error("Type mismatch")]
    TypeMismatch,
}

pub(crate) struct InstructionDecoder {
    storage: InstructionStorage,
}

impl InstructionDecoder {
    pub(crate) fn new(storage: InstructionStorage) -> InstructionDecoder {
        InstructionDecoder { storage }
    }

    pub(crate) fn read<I>(&mut self, t: InstructionType) -> Result<I, DecodingError>
    where
        I: Instruction,
    {
        I::deserialize(self, t)
    }

    pub(crate) fn read_instruction_type(&mut self) -> Result<InstructionType, DecodingError> {
        self.storage
            .instruction_storage
            .pop_front()
            .ok_or(DecodingError::DecodingError(
                "instruction storage exhausted".to_string(),
            ))
    }

    pub(crate) fn read_immediate<T: Integer>(&mut self) -> Result<T, DecodingError> {
        // TODO: test if it is sufficient to only call this once on construction or whether a pop_front might make the content non-contiguous
        let drained_bytes = self
            .storage
            .immediate_storage
            .drain(..std::mem::size_of::<T>());
        Ok(T::from_le_bytes(
            drained_bytes.collect::<Vec<u8>>().as_slice(),
        ))
    }

    pub(crate) fn read_immediate_float32(&mut self) -> Result<f32, DecodingError> {
        self.storage.immediate_storage.make_contiguous();
        let drained_bytes = self
            .storage
            .immediate_storage
            .drain(..std::mem::size_of::<f64>());
        let byte_arr =
            <[u8; 4]>::try_from(drained_bytes.collect::<Vec<u8>>().as_slice()).map_err(|_| {
                DecodingError::DecodingError("Failed to find all bytes for float parsing".into())
            })?;
        let imm = f32::from_le_bytes(byte_arr);
        Ok(imm)
    }

    pub(crate) fn read_immediate_float64(&mut self) -> Result<f64, DecodingError> {
        self.storage.immediate_storage.make_contiguous();
        let drained_bytes = self
            .storage
            .immediate_storage
            .drain(..std::mem::size_of::<f64>());
        let byte_arr =
            <[u8; 8]>::try_from(drained_bytes.collect::<Vec<u8>>().as_slice()).map_err(|_| {
                DecodingError::DecodingError("Failed to find all bytes for float parsing".into())
            })?;
        let imm = f64::from_le_bytes(byte_arr);
        Ok(imm)
    }

    pub(crate) fn read_terminator(&self) -> ControlInstruction {
        self.storage.terminator.clone()
    }

    pub(crate) fn read_value_type(&mut self) -> Result<ValType, DecodingError> {
        self.storage
            .type_storage
            .pop_front()
            .ok_or(DecodingError::DecodingError(
                "value type storage exhausted".to_string(),
            ))
    }

    pub(crate) fn read_variable(&mut self) -> Result<VariableID, DecodingError> {
        self.storage
            .variable_storage
            .pop_front()
            .ok_or(DecodingError::DecodingError(
                "variable storage exhausted".to_string(),
            ))
    }
}
