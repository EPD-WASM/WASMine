use super::stack::ParserStack;
use crate::ValidationError;
use module::{instructions::Variable, ModuleMetadata};
use std::sync::atomic::{AtomicUsize, Ordering};
use wasm_types::ValType;

pub struct Context<'a> {
    pub(crate) module: &'a ModuleMetadata,
    pub(crate) stack: ParserStack,
    pub(crate) locals: Vec<ValType>,
    pub(crate) var_count: AtomicUsize,
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

    pub fn create_var(&self, type_: ValType) -> Variable {
        Variable {
            type_,
            id: self.var_count.fetch_add(1, Ordering::Relaxed),
        }
    }

    pub(crate) fn new(module: &'a ModuleMetadata, locals: Vec<ValType>) -> Self {
        Self {
            module,
            stack: ParserStack::new(),
            locals,
            var_count: AtomicUsize::new(0),
            poison: None,
        }
    }

    // you may now ask yourself: why are stack functions in the context?
    // glad you ask. we need to mutate the ctxt for the poisining AND the stack modification.
    // That's way easier if we have the stack functions in the context.
    pub fn push_var(&mut self, var: Variable) {
        self.stack.push_var(var)
    }

    pub(crate) fn pop_var_with_type(&mut self, type_: ValType) -> Variable {
        let pop_result = self.stack.pop_var_with_type(type_);
        self.extract_poison(pop_result)
    }

    pub fn pop_var(&mut self) -> Variable {
        let pop_result = self.stack.pop_var();
        self.extract_poison(pop_result)
    }
}
