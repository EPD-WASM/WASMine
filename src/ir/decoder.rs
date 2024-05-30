use super::basic_block::BasicBlockStorage;
use crate::{instructions::*, util::integer_traits::Integer};
use thiserror::Error;
use wasm_types::{InstructionType, ValType};

#[derive(Debug, Error)]
pub enum DecodingError {
    #[error("Decoding error: {0}")]
    DecodingError(String),
    #[error("Type mismatch")]
    TypeMismatch,
}

#[derive(Debug)]
pub(crate) struct InstructionDecoder {
    storage: BasicBlockStorage,
}

impl InstructionDecoder {
    pub(crate) fn new(storage: BasicBlockStorage) -> InstructionDecoder {
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
        debug_assert!(self.storage.immediate_storage.len() >= std::mem::size_of::<T>());
        let drained_bytes = self
            .storage
            .immediate_storage
            .drain(..std::mem::size_of::<T>());
        Ok(T::from_bytes(drained_bytes.collect::<Vec<u8>>().as_slice()))
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
