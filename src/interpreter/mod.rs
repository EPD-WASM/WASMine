use std::collections::HashMap;

use thiserror::Error;

use crate::{
    instructions::VariableID,
    ir::{DecodingError, InstructionDecoder},
    parser::ParserError,
    structs::{module::Module, value::Value},
};
use wasm_types::{FuncIdx, InstructionType};

mod numeric;

#[derive(Debug, Error)]
pub(crate) enum InterpreterError {
    #[error("Execution error: {0}")]
    ExecutionError(String),
    #[error("Type mismatch")]
    TypeMismatch,
    #[error("Type not valid for this operation")]
    InvalidType,
    #[error("Decoding error: {0}")]
    DecodingError(#[from] DecodingError),
    #[error("Parser error: {0}")]
    ParserError(#[from] ParserError),
}

pub(crate) trait Executable {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError>;
}

#[derive(Debug)]
pub(crate) struct LocalVariableStore {
    vars: Vec<u64>,
}

impl LocalVariableStore {
    pub(crate) fn new() -> Self {
        Self { vars: Vec::new() }
    }

    // TODO bounds check? => if parser is implemented correctly, this should not be necessary (maybe debug assert?)
    pub(crate) fn get(&self, idx: VariableID) -> u64 {
        debug_assert!((idx as usize) < self.vars.len());
        self.vars[idx as usize]
    }

    pub(crate) fn set(&mut self, idx: VariableID, value: u64) {
        // TODO: store highest variable id to avoid resizing the vector all the time
        if idx as usize >= self.vars.len() {
            self.vars.resize(idx as usize + 1, 0);
        }
        self.vars[idx as usize] = value;
    }
}

// (also) stores indices into each of the instruction store's fields
#[derive(Debug)]
struct StackFrame {
    /// function index
    fn_idx: FuncIdx,
    /// basic block index (within the function)
    bb_idx: u32,
    /// the instruction decoder used to read out instructions
    decoder: InstructionDecoder,
    vars: LocalVariableStore,
}

pub(crate) struct InterpreterContext {
    module: Module,
    variables: HashMap<VariableID, Value>,
    stack: Vec<StackFrame>,
}

impl InterpreterContext {
    fn new(module: Module) -> Self {
        Self {
            module,
            variables: HashMap::new(),
            stack: Vec::new(),
        }
    }
}

struct Interpreter {
    ctx: InterpreterContext,
}

impl Interpreter {
    fn new(context: InterpreterContext) -> Self {
        Self { ctx: context }
    }

    fn run(&mut self) -> Result<(), InterpreterError> {
        if self.ctx.stack.is_empty() {
            let entry_fn_idx: FuncIdx = self.ctx.module.entry_point.unwrap_or(0_u32);
            let entry_fn = self
                .ctx
                .module
                .ir
                .functions
                .get(entry_fn_idx as usize)
                // TODO: make this check only for debug builds. We KNOW that this function exists
                .expect(format!("Function not found at index {entry_fn_idx}").as_str());

            // TODO: store pointer to entry block. This is currently always BB0, but this might change in the future
            let basic_block = entry_fn
                .basic_blocks
                .get(0)
                // TODO: make this check only for debug builds. We KNOW that this basic block exists
                .expect("Basic block not found at index 0");

            let decoder = InstructionDecoder::new(basic_block.instructions.clone());
            self.ctx.stack.push(StackFrame {
                fn_idx: entry_fn_idx,
                bb_idx: 0,
                decoder,
                vars: LocalVariableStore::new(),
            });
        }

        let current_frame = self.ctx.stack.last_mut().unwrap();
        let instruction_type = current_frame.decoder.read_instruction_type()?;
        match instruction_type.clone() {
            InstructionType::Numeric(c) => numeric::execute_numeric_instruction(
                &mut self.ctx,
                c,
                // &mut current_frame.decoder,
                instruction_type,
            )?,
            InstructionType::Vector(_) => todo!(),
            InstructionType::Parametric(_) => todo!(),
            InstructionType::Variable(_) => todo!(),
            InstructionType::Table(_) => todo!(),
            InstructionType::Memory(_) => todo!(),
            InstructionType::Control(_) => todo!(),
            InstructionType::Reference(_) => todo!(),
            InstructionType::Meta(_) => todo!(),
        }

        todo!()
    }
}
