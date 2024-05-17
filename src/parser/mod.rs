mod error;
mod opcode_tbl;
pub(crate) mod parsable;
mod parse_basic_blocks;
#[allow(clippy::module_inception)]
pub mod parser;
pub(crate) mod wasm_stream_reader;

pub(crate) use self::error::{ParserError, ValidationError};

use crate::{
    instructions::Variable,
    structs::{function::Function, module::Module},
    wasm_types::ValType,
};
use std::sync::atomic::{AtomicU32, Ordering};

pub(crate) type ParseResult = Result<(), ParserError>;

#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct ParserStack {
    pub(crate) stack: Vec<Variable>,
}

pub(crate) struct Context<'a> {
    pub(crate) module: &'a Module,
    pub(crate) stack: ParserStack,
    pub(crate) func: &'a Function,
    pub(crate) var_count: AtomicU32,
    pub(crate) poison: Option<ValidationError>,
}

impl<'a> Context<'a> {
    pub(crate) fn poison<V: Default>(&mut self, err: ValidationError) -> V {
        if self.poison.is_none() {
            self.poison = Some(err);
        }
        Default::default()
    }

    pub(crate) fn extract_poison<V: Default>(&mut self, result: Result<V, ValidationError>) -> V {
        match result {
            Ok(v) => v,
            Err(e) => self.poison(e),
        }
    }

    pub(crate) fn create_var(&self, type_: ValType) -> Variable {
        Variable {
            type_,
            id: self.var_count.fetch_add(1, Ordering::Relaxed),
        }
    }

    pub(crate) fn new(module: &'a Module, func: &'a Function) -> Self {
        Self {
            module,
            stack: ParserStack::new(),
            func,
            var_count: AtomicU32::new(0),
            poison: None,
        }
    }

    // you may now ask yourself: why are stack functions in the context?
    // glad you ask. we need to mutate the ctxt for the poisining AND the stack modification.
    // That's way easier if we have the stack functions in the context.
    pub(crate) fn push_var(&mut self, var: Variable) {
        self.stack.push_var(var)
    }

    pub(crate) fn pop_var_with_type(&mut self, type_: &ValType) -> Variable {
        let pop_result = self.stack.pop_var_with_type(type_);
        self.extract_poison(pop_result)
    }

    pub(crate) fn pop_var(&mut self) -> Variable {
        let pop_result = self.stack.pop_var();
        self.extract_poison(pop_result)
    }
}

impl ParserStack {
    pub(crate) fn new() -> Self {
        Self { stack: Vec::new() }
    }

    fn push_var(&mut self, var: Variable) {
        self.stack.push(var)
    }

    fn pop_var_with_type(&mut self, type_: &ValType) -> Result<Variable, ValidationError> {
        let var = match self.stack.pop() {
            Some(var) => var,
            None => return Err(ValidationError::Msg("stack underflow".into())),
        };
        if var.type_ != *type_ {
            return Err(ValidationError::Msg("type mismatch".into()));
        }
        Ok(var)
    }

    fn pop_var(&mut self) -> Result<Variable, ValidationError> {
        self.stack
            .pop()
            .ok_or(ValidationError::Msg("stack underflow".into()))
    }
}
