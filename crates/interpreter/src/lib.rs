#![allow(clippy::all)]
#![allow(warnings)]
use control_flow::GlueHandler;
use core::ffi;
use module::{
    basic_block::BasicBlockGlue,
    instructions::FunctionIR,
    objects::{
        function::{Function, FunctionImport, FunctionSource},
        value::{Number, ValueRaw},
    },
    utils::numeric_transmutes::{Bit32, Bit64},
};
use runtime_interface::{ExecutionContext, GlobalInstance, RawPointer};
use std::{collections::HashMap, fmt::Display, rc::Rc};
use table::execute_table_instruction;
use thiserror::Error;
use wasm_types::{FuncIdx, GlobalIdx, InstructionType, NumType, ValType};
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
mod variable;

use log;

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
    #[error("Division by zero")]
    DivZero,
    #[error("Error converting float to integer")]
    TruncError,
    #[error{"Global at index {0} not found"}]
    GlobalNotFound(GlobalIdx),
    #[error{"Stack exhausted"}]
    StackExhausted,
    #[error{"No IR in module"}]
    NoIR,
    #[error{"Function at index {0} not found"}]
    FunctionNotFound(FuncIdx),
}

pub(crate) trait Executable {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError>;
}

#[derive(Debug)]
pub(crate) struct VariableStore {
    vars: Vec<ValueRaw>,
}

impl VariableStore {
    pub(crate) fn new(init: Vec<ValueRaw>) -> Self {
        Self { vars: init }
    }

    pub(crate) fn get(&self, idx: VariableID) -> ValueRaw {
        log::trace!("getting from idx: {}. Value: ", idx);
        let value_raw = self
            .vars
            .get(idx as usize)
            .unwrap_or(&ValueRaw::u64(0))
            .clone();
        log::trace!("{}", unsafe { value_raw.as_u64() });
        value_raw
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

    pub(crate) fn get_number_signed(
        &self,
        idx: VariableID,
        num_ty: NumType,
        signed: bool,
    ) -> Number {
        let n = match self.get_value(idx, ValType::Number(num_ty)) {
            Value::Number(n) => n,
            _ => unreachable!(),
        };

        if signed {
            n.as_signed()
        } else {
            n.as_unsigned()
        }
    }

    pub(crate) fn set(&mut self, idx: VariableID, value: ValueRaw) {
        log::trace!("setting idx: {} to value: {}", idx, unsafe {
            value.as_u64()
        });
        log::trace!("                        = {}", unsafe {
            value.as_f32().trans_f32()
        });
        log::trace!("                        = {}", unsafe {
            value.as_f64().trans_f64()
        });

        if idx as usize >= self.vars.len() {
            log::trace!("resizing vars to len: {}", idx as usize + 1);
            self.vars.resize(idx as usize + 1, ValueRaw::v128([0; 16]));
        }
        self.vars[idx as usize] = value;
    }
}

impl Display for VariableStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[\n{}\n]",
            self.vars
                .iter()
                .enumerate()
                .map(|(i, n)| (i, unsafe { n.as_u64() }))
                .map(|(i, n)| format!("{}: {}", i, n))
                .collect::<Vec<String>>()
                .join(",\n")
        )
    }
}

// (also) stores indices into each of the instruction store's fields
#[derive(Debug)]
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
    was_imported_terminator: bool,
}

type ImportMap = HashMap<String, RawPointer>;
type GlobalMap = HashMap<GlobalIdx, RawPointer>;

pub struct InterpreterContext<'a> {
    module: Rc<Module>,
    stack: Vec<StackFrame>,
    exec_ctx: &'a mut ExecutionContext,
    imported_symbols: ImportMap,
    ir: Rc<Vec<FunctionIR>>,
}

pub enum InterpreterFunc<'a> {
    IR(&'a FunctionIR),
    Import(u32),
}

impl<'a> InterpreterContext<'a> {
    pub fn new(
        module_rc: Rc<Module>,
        exec_ctx: &'a mut ExecutionContext,
        imported_symbols: ImportMap,
        ir: Rc<Vec<FunctionIR>>,
    ) -> Self {
        Self {
            module: module_rc,
            stack: Vec::new(),
            exec_ctx,
            imported_symbols,
            ir,
        }
    }
}

#[derive(Clone)]
pub struct Interpreter {
    module: Option<Rc<Module>>,
    ir: Option<Rc<Vec<FunctionIR>>>,
    pub imported_functions: ImportMap,
    global_addresses: GlobalMap,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            module: None,
            imported_functions: HashMap::new(),
            global_addresses: HashMap::new(),
            ir: None,
        }
    }

    pub fn set_module(&mut self, module: Rc<Module>) {
        self.module = Some(module);
    }

    pub fn set_ir(&mut self, ir: Rc<Vec<FunctionIR>>) {
        self.ir = Some(ir);
    }

    pub fn set_symbol_addr(&mut self, name: &str, address: RawPointer) {
        self.imported_functions.insert(name.to_string(), address);
    }

    pub fn get_symbol_addr(&self, name: &str) -> Option<&RawPointer> {
        self.imported_functions.get(name)
    }

    pub fn set_global_addr(&mut self, idx: GlobalIdx, address: RawPointer) {
        self.global_addresses.insert(idx, address);
    }

    pub fn get_global_value(&self, idx: GlobalIdx) -> Result<ValueRaw, InterpreterError> {
        let global_addr = self
            .global_addresses
            .get(&idx)
            .ok_or(InterpreterError::GlobalNotFound(idx))?;
        let global_instance: GlobalInstance =
            unsafe { std::ptr::read(global_addr.as_ptr() as *const GlobalInstance) };

        let global_value = unsafe { std::ptr::read(global_instance.addr.as_ptr()) };

        Ok(global_value)
    }

    pub unsafe fn run(
        &mut self,
        function_idx: FuncIdx,
        parameters: Vec<Value>,
        exec_ctx: *mut ExecutionContext,
    ) -> Result<Vec<Value>, InterpreterError> {
        log::trace!("Module: {:#?}", self.module.as_ref().unwrap().meta);

        log::info!(
            " ===== Interpreter running function: idx: {} =====",
            function_idx,
        );

        let fn_name =
            Function::debug_function_name(function_idx, &self.module.as_ref().unwrap().meta);
        log::info!("Function name: {}", fn_name);

        let exec_ctx = unsafe { exec_ctx.as_mut().unwrap() };

        let mut ctx = InterpreterContext::new(
            self.module.clone().unwrap(),
            exec_ctx,
            self.imported_functions.clone(),
            self.ir.clone().unwrap(),
        );

        ctx.stack = Vec::new();

        let entry_fn_meta = ctx
            .module
            .meta
            .functions
            .get(function_idx as usize)
            .unwrap_or_else(|| panic!("Function not found at index {function_idx}."));

        let raw_parameters = parameters.into_iter().map(|v| v.into()).collect::<Vec<_>>();

        let fn_type = ctx.module.meta.function_types[entry_fn_meta.type_idx as usize];
        let ret_types = fn_type.results();
        log::info!("Function signature: {}", fn_type);
        log::info!("Recursion size: {}", ctx.exec_ctx.recursion_size);
        log::trace!("entry fn: {:#?}", &entry_fn_meta);

        ctx.exec_ctx.recursion_size += 1;

        let entry_fn_res: Result<_, InterpreterError> = {
            let ir: &Vec<FunctionIR> = &ctx.ir;

            let fn_meta = match (&ctx).module.meta.functions.get(function_idx as usize) {
                Some(meta) => meta,
                None => return Err(InterpreterError::FunctionNotFound(function_idx)),
            };

            match &fn_meta.source {
                FunctionSource::Import(FunctionImport { import_idx }) => {
                    Ok(InterpreterFunc::Import(*import_idx))
                }
                FunctionSource::Wasm(_) => Ok(InterpreterFunc::IR(&ir[function_idx as usize])),
            }
        };

        let entry_fn = entry_fn_res?;

        let ret_vals = match entry_fn {
            InterpreterFunc::Import(import_idx) => {
                let import_idx = import_idx as usize;

                control_flow::util::call_import_helper(&mut ctx, import_idx, &raw_parameters)
            }

            InterpreterFunc::IR(entry_fn) => {
                let bbs = &entry_fn.bbs;

                let basic_block = bbs.first().unwrap();

                let decoder = InstructionDecoder::new(basic_block.instructions.clone());

                ctx.stack.push(StackFrame {
                    fn_idx: function_idx,
                    fn_local_vars: VariableStore::new(raw_parameters),
                    bb_id: basic_block.id,
                    last_bb_id: 0,
                    return_vars: Vec::new(),
                    decoder,
                    vars: VariableStore::new(Vec::new()),
                    was_imported_terminator: false,
                });

                loop {
                    // check for call stack exhaustion
                    if ctx.exec_ctx.recursion_size > 100_000 {
                        ctx.exec_ctx.recursion_size = 0;
                        return Err(InterpreterError::StackExhausted);
                    }

                    log::trace!("recursion_size: {}", ctx.exec_ctx.recursion_size);
                    log::trace!("stack len: {}", &ctx.stack.len());

                    let instruction_type = ctx
                        .stack
                        .last_mut()
                        .unwrap()
                        .decoder
                        .read_instruction_type();

                    log::debug!("Instruction type: {:?}", instruction_type);

                    match instruction_type {
                        Ok(instruction_type) => {
                            self.execute_instruction(instruction_type, &mut ctx)?
                        }
                        Err(DecodingError::InstructionStorageExhausted) => {
                            let current_fn_idx = ctx.stack.last_mut().unwrap().fn_idx;
                            let interpreter_func_res: Result<_, InterpreterError> = {
                                let ir: &Vec<FunctionIR> = &ctx.ir;

                                let fn_meta =
                                    match (&ctx).module.meta.functions.get(current_fn_idx as usize)
                                    {
                                        Some(meta) => meta,
                                        None => {
                                            return Err(InterpreterError::FunctionNotFound(
                                                current_fn_idx,
                                            ))
                                        }
                                    };

                                match &fn_meta.source {
                                    FunctionSource::Import(FunctionImport { import_idx }) => {
                                        Ok(InterpreterFunc::Import(*import_idx))
                                    }
                                    FunctionSource::Wasm(_) => {
                                        Ok(InterpreterFunc::IR(&ir[current_fn_idx as usize]))
                                    }
                                }
                            };
                            let func = match interpreter_func_res.unwrap() {
                                InterpreterFunc::IR(function_ir) => function_ir,
                                InterpreterFunc::Import(_) => unreachable!(),
                            };
                            let bbs = &func.bbs;

                            let current_frame = ctx.stack.last_mut().unwrap();

                            let bb = bbs
                                .iter()
                                .find(|bb| bb.id == current_frame.bb_id)
                                .unwrap_or_else(|| {
                                    panic!("Basic block with ID {} not found", current_frame.bb_id)
                                });

                            log::debug!("handling terminator {:?}", bb.terminator);
                            log::trace!("from basic block: {:#?}", bb);

                            let ret_vals = bb.terminator.to_owned().handle(&mut ctx)?;

                            let should_pop = ctx
                                .stack
                                .last_mut()
                                .map_or(false, |sf| sf.was_imported_terminator);
                            if should_pop {
                                ctx.stack.pop();
                            }

                            log::debug!("stack len: {}", &ctx.stack.len());

                            if ctx.stack.len() == 0 {
                                let mut ret_vals = ret_vals.unwrap_or_default();
                                ret_vals.truncate(fn_type.num_results());

                                break ret_vals;
                            }
                        }
                        Err(e) => return Err(InterpreterError::DecodingError(e)),
                    }
                }
            }
        };

        debug_assert_eq!(
            ret_types.len(),
            ret_vals.len(),
            "\nExpected return types:\t{:?},\n\t\t\tgot values:\t{:?}",
            ret_types,
            ret_vals.iter().map(|v| v.as_u64()).collect::<Vec<_>>()
        );

        let ret_vals = ret_vals
            .into_iter()
            .zip(ret_types.iter())
            .map(|(val, ty)| Value::from_raw(val, *ty))
            .collect();

        log::info!(
            " ===== Function {} (idx: {}) returned: {:?} =====\n\n",
            fn_name,
            function_idx,
            &ret_vals
        );

        ctx.exec_ctx.recursion_size -= 1;

        Ok(ret_vals)
    }

    fn execute_instruction(
        &mut self,
        instruction_type: InstructionType,
        ctx: &mut InterpreterContext,
    ) -> Result<(), InterpreterError> {
        log::debug!("Interpreting instruction: {:?}", instruction_type);
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
