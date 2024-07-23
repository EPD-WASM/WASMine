use super::basic_block::BasicBlockStorage;
use crate::{
    instructions::{Instruction, PhiNode, VariableID},
    structs::instruction::ControlInstruction,
    utils::integer_traits::Integer,
};
use std::collections::VecDeque;
use wasm_types::{InstructionType, ValType};

#[derive(Clone)]
pub struct InstructionEncoder {
    storage: BasicBlockStorage,
    finished: bool,
}

impl InstructionEncoder {
    pub fn new() -> InstructionEncoder {
        Self::default()
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }

    pub fn write<I>(&mut self, i: I)
    where
        I: Instruction,
    {
        i.serialize(self);
    }

    pub fn write_instruction_type(&mut self, type_: InstructionType) {
        self.storage.instruction_storage.push_back(type_);
    }

    pub fn write_immediate<T: Integer>(&mut self, imm: T) {
        // TODO: align storage for better performance
        self.storage.immediate_storage.extend(imm.to_bytes());
    }

    pub fn write_variable(&mut self, var: VariableID) {
        self.storage.variable_storage.push_back(var);
    }

    pub fn write_value_type(&mut self, type_: ValType) {
        self.storage.type_storage.push_back(type_);
    }

    pub fn finish(&mut self, terminator: ControlInstruction) {
        self.storage.terminator = terminator;
        self.finished = true;
    }

    pub fn add_input(&mut self, phi: PhiNode) {
        self.storage.inputs.push(phi);
    }

    pub fn extract_data(self) -> BasicBlockStorage {
        self.storage
    }

    pub fn peek_terminator(&self) -> &ControlInstruction {
        &self.storage.terminator
    }
}

impl Default for InstructionEncoder {
    fn default() -> Self {
        InstructionEncoder {
            storage: BasicBlockStorage {
                immediate_storage: VecDeque::new(),
                variable_storage: VecDeque::new(),
                type_storage: VecDeque::new(),
                instruction_storage: VecDeque::new(),
                terminator: ControlInstruction::Unreachable,
                inputs: Vec::new(),
            },
            finished: false,
        }
    }
}
