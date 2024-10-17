use super::basic_block::BasicBlockStorage;
use crate::{instructions::*, utils::integer_traits::Integer};
use thiserror::Error;
use wasm_types::{InstructionType, ValType};

#[derive(Debug, Error, Clone)]
pub enum DecodingError {
    #[error("Decoding error: {0}")]
    DecodingError(String),
    #[error("Type mismatch")]
    TypeMismatch,
    #[error("Instruction storage exhausted")]
    InstructionStorageExhausted,
    #[error("value type storage exhausted")]
    ValueTypeStorageExhausted,
    #[error("variable storage exhausted")]
    VariableStorageExhausted,
}

#[derive(Debug)]
pub struct InstructionDecoder {
    storage: BasicBlockStorage,
}

impl InstructionDecoder {
    pub fn new(storage: BasicBlockStorage) -> InstructionDecoder {
        InstructionDecoder { storage }
    }

    pub fn read<I>(&mut self, t: InstructionType) -> Result<I, DecodingError>
    where
        I: Instruction,
    {
        I::deserialize(self, t)
    }

    pub fn read_instruction_type(&mut self) -> Result<InstructionType, DecodingError> {
        self.storage
            .instruction_storage
            .pop_front()
            .ok_or(DecodingError::InstructionStorageExhausted)
    }

    pub fn read_immediate<T: Integer>(&mut self) -> Result<T, DecodingError> {
        let size = std::mem::size_of::<T>();
        debug_assert!(self.storage.immediate_storage.len() >= size);
        // we can do this because the VecDeque is made contiguous upon completion of the block,
        // so the first slice is the entire buffer. This seems to be faster than simply using
        // the drain return value, likely because the compiler knows we're operating only on the first slice.
        let bytes = &self.storage.immediate_storage.as_slices().0[..size];
        let ret = Ok(T::from_bytes(bytes));
        self.storage.immediate_storage.drain(0..size);
        ret
    }

    pub fn read_value_type(&mut self) -> Result<ValType, DecodingError> {
        self.storage
            .type_storage
            .pop_front()
            .ok_or(DecodingError::ValueTypeStorageExhausted)
    }

    pub fn read_variable(&mut self) -> Result<VariableID, DecodingError> {
        self.storage
            .variable_storage
            .pop_front()
            .ok_or(DecodingError::VariableStorageExhausted)
    }
}
