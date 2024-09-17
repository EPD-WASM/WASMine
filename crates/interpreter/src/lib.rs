#![allow(clippy::all)]
#![allow(warnings)]
use control_flow::GlueHandler;
use core::ffi;
use module::objects::value::{Number, ValueRaw};
use runtime_interface::{ExecutionContext, RawPointer};
use std::{collections::HashMap, rc::Rc};
use table::execute_table_instruction;
use thiserror::Error;
use wasm_types::{FuncIdx, InstructionType, NumType, ValType};
use {
    module::instructions::VariableID,
    module::objects::{module::Module, value::Value},
    module::{DecodingError, InstructionDecoder},
    parser::error::ParserError,
};

mod control_flow;
mod memory;
mod numeric;
mod parametric;
mod reference;
mod table;
mod util;
mod variable;

#[derive(Debug, Error)]
pub enum InterpreterError {
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
    #[error("Unreachable instruction reached")]
    Unreachable,
    #[error("Index out of bounds")]
    IdxBounds,
}

pub(crate) trait Executable {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError>;
}

pub(crate) struct VariableStore {
    vars: Vec<ValueRaw>,
}

impl VariableStore {
    pub(crate) fn new(init: Vec<ValueRaw>) -> Self {
        Self { vars: init }
    }

    // TODO bounds check? => if parser is implemented correctly, this should not be necessary (maybe debug assert?)
    pub(crate) fn get(&self, idx: VariableID) -> ValueRaw {
        self.vars[idx as usize]
    }

    pub(crate) fn get_value(&self, idx: VariableID, val_type: ValType) -> Value {
        Value::from_raw(self.vars[idx as usize], val_type)
    }

    pub(crate) fn get_number(&self, idx: VariableID, num_ty: NumType) -> Number {
        match self.get_value(idx, ValType::Number(num_ty)) {
            Value::Number(n) => n,
            _ => unreachable!(),
        }
    }

    pub(crate) fn set(&mut self, idx: VariableID, value: ValueRaw) {
        // TODO: store highest variable id to avoid resizing the vector all the time
        // println!("setting idx: {} to value: {}", idx, value);
        // println!("                        = {}", value.trans_f64());

        if idx as usize >= self.vars.len() {
            // println!("resizing vars to len: {}", idx as usize + 1);
            self.vars.resize(idx as usize + 1, ValueRaw::i32(0));
        }
        // println!("\tvars: {:?}", self.vars);
        self.vars[idx as usize] = value;
        // println!("--> \tvars: {:?}", self.vars);
        // println!(
        //     " == \tvars: {:?}",
        //     self.vars
        //         .iter()
        //         .map(|n| n.trans_f64())
        //         .collect::<Vec<f64>>()
        // );
    }
}

// (also) stores indices into each of the instruction store's fields
struct StackFrame {
    /// function index
    fn_idx: FuncIdx,
    /// local variables of the function
    fn_local_vars: VariableStore,
    /// basic block index (within the function)
    bb_id: u32,
    last_bb_id: u32,
    /// indices into which the return values are written in the caller function
    return_vars: Vec<VariableID>,
    /// the instruction decoder used to read out instructions
    decoder: InstructionDecoder,
    vars: VariableStore,
}

pub struct InterpreterContext<'a> {
    module: Rc<Module>,
    stack: Vec<StackFrame>,
    exec_ctx: &'a mut ExecutionContext,
}

impl<'a> InterpreterContext<'a> {
    pub fn new(module_rc: Rc<Module>, exec_ctx: &'a mut ExecutionContext) -> Self {
        Self {
            module: module_rc,
            stack: Vec::new(),
            exec_ctx,
        }
    }
}

pub struct Interpreter {
    // ctx: InterpreterContext,
    module: Option<Rc<Module>>,
    imported_functions: HashMap<String, RawPointer>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            module: None,
            imported_functions: HashMap::new(),
        }
    }

    pub fn set_module(&mut self, module: Rc<Module>) {
        self.module = Some(module);
    }

    pub fn register_symbol(&mut self, name: &str, address: RawPointer) {
        self.imported_functions.insert(name.to_string(), address);
    }

    pub unsafe fn run(
        &mut self,
        function_idx: FuncIdx,
        parameters: Vec<Value>,
        exec_ctx: *mut ExecutionContext,
    ) -> Result<Vec<Value>, InterpreterError> {
        // println!("Interpreter running function: {}", function_idx);
        let exec_ctx = unsafe { exec_ctx.as_mut().unwrap() };

        let mut ctx = InterpreterContext::new(self.module.clone().unwrap(), exec_ctx);

        ctx.stack = Vec::new();
        let entry_fn = ctx
            .module
            .meta
            .functions
            .get(function_idx as usize)
            // TODO: make this check only for debug builds. We KNOW that this function exists
            .unwrap_or_else(|| panic!("Function not found at index {function_idx}"));

        // TODO: store pointer to entry block. This is currently always BB0, but this might change in the future
        let basic_block = util::get_bbs_from_function(entry_fn).first().unwrap();

        let decoder = InstructionDecoder::new(basic_block.instructions.clone());

        let raw_parameters = parameters.into_iter().map(|v| v.into()).collect::<Vec<_>>();

        ctx.stack.push(StackFrame {
            fn_idx: function_idx,
            fn_local_vars: VariableStore::new(raw_parameters),
            bb_id: basic_block.id,
            last_bb_id: 0,
            return_vars: Vec::new(),
            decoder,
            vars: VariableStore::new(Vec::new()),
        });

        let ret_types = ctx.module.meta.function_types[entry_fn.type_idx as usize].results();
        // println!("return types: {:?}", ret_types);
        // println!("decoded instructions: {:#?}", self.ctx.stack.last().unwrap().decoder);
        // println!("entry_fn: {:#?}", entry_fn);

        let ret_vals = loop {
            let instruction_type = ctx
                .stack
                .last_mut()
                .unwrap()
                .decoder
                .read_instruction_type();

            // println!("{:?}", instruction_type);

            match instruction_type {
                Ok(instruction_type) => self.execute_instruction(instruction_type, &mut ctx)?,
                Err(DecodingError::InstructionStorageExhausted) => {
                    // println!("stack: {:#?}", self.ctx.stack);

                    let current_frame = ctx.stack.last_mut().unwrap();
                    let func = &ctx.module.meta.functions[current_frame.fn_idx as usize];
                    let bbs = util::get_bbs_from_function(func);
                    // println!("func basic blocks: {:#?}", func.basic_blocks);

                    let bb = bbs
                        .iter()
                        .find(|bb| bb.id == current_frame.bb_id)
                        .unwrap_or_else(|| {
                            panic!("Basic block with ID {} not found", current_frame.bb_id)
                        });

                    // println!("handling terminator {:?}", bb.terminator);

                    let ret_vals = bb.terminator.to_owned().handle(&mut ctx)?;
                    if ctx.stack.is_empty() {
                        break ret_vals.unwrap_or_default();
                    }

                    // break self.ctx.last_return_values.clone();
                }
                Err(e) => return Err(InterpreterError::DecodingError(e)),
            }
        };

        let ret_vals = ret_vals
            .into_iter()
            .enumerate()
            .map(|(i, val)| Value::from_raw(val, ret_types[i]))
            .collect();

        Ok(ret_vals)
    }

    fn execute_instruction(
        &mut self,
        instruction_type: InstructionType,
        ctx: &mut InterpreterContext,
    ) -> Result<(), InterpreterError> {
        // println!("Interpreting instruction: {:?}", instruction_type);
        match instruction_type.clone() {
            InstructionType::Numeric(c) => {
                numeric::execute_numeric_instruction(ctx, c, instruction_type)
            }
            InstructionType::Variable(c) => {
                variable::execute_variable_instruction(ctx, c, instruction_type)
            }
            InstructionType::Parametric(c) => {
                parametric::execute_parametric_instruction(ctx, c, instruction_type)
            }
            InstructionType::Memory(c) => {
                memory::execute_memory_instruction(ctx, c, instruction_type)
            }
            InstructionType::Meta(_) => unreachable!("No meta instructions exist"),

            InstructionType::Reference(c) => {
                reference::execute_reference_instruction(ctx, c, instruction_type)
            }
            InstructionType::Table(c) => execute_table_instruction(ctx, c, instruction_type),
            InstructionType::Control(_) => unreachable!(
                "Control instructions are not serialized and can therefore not be deserialized."
            ),
            InstructionType::Vector => todo!(),
        }
    }
}
