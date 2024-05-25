use super::InstructionStorage;
use crate::{
    instructions::{Instruction, VariableID},
    structs::instruction::ControlInstruction,
    util::integer_traits::Integer,
};
use std::collections::VecDeque;
use wasm_types::{InstructionType, ValType};

pub(crate) struct InstructionEncoder {
    storage: InstructionStorage,
    finished: bool,
}

impl InstructionEncoder {
    pub(crate) fn new() -> InstructionEncoder {
        Self::default()
    }

    pub(crate) fn is_finished(&self) -> bool {
        self.finished
    }

    pub(crate) fn write<I>(&mut self, i: I)
    where
        I: Instruction,
    {
        i.serialize(self);
    }

    pub(crate) fn write_instruction_type(&mut self, type_: InstructionType) {
        self.storage.instruction_storage.push_back(type_);
    }

    pub(crate) fn write_immediate<T: Integer>(&mut self, imm: T) {
        // TODO: align storage for better performance
        self.storage.immediate_storage.extend(&imm.to_bytes());
    }

    pub(crate) fn write_variable(&mut self, var: VariableID) {
        self.storage.variable_storage.push_back(var);
    }

    pub(crate) fn write_value_type(&mut self, type_: ValType) {
        self.storage.type_storage.push_back(type_);
    }

    pub(crate) fn finish(&mut self, terminator: ControlInstruction) {
        self.storage.terminator = terminator;
        self.finished = true;
    }

    pub(crate) fn extract_data(self) -> InstructionStorage {
        self.storage
    }
}

impl Default for InstructionEncoder {
    fn default() -> Self {
        InstructionEncoder {
            storage: InstructionStorage {
                immediate_storage: VecDeque::new(),
                variable_storage: VecDeque::new(),
                type_storage: VecDeque::new(),
                instruction_storage: VecDeque::new(),
                terminator: ControlInstruction::Unreachable,
            },
            finished: false,
        }
    }
}
