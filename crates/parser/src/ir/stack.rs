use std::ops::Index;

use module::instructions::Variable;
use wasm_types::ValType;

use crate::ValidationError;

#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct ParserStack {
    pub(crate) stack: Vec<Variable>,
    pub(crate) stash: Vec<usize>,
}

impl ParserStack {
    pub(crate) fn new() -> Self {
        Self {
            stack: Vec::new(),
            stash: vec![0],
        }
    }

    pub(crate) fn stash(&mut self) {
        self.stash.push(self.stack.len());
    }

    pub(crate) fn stash_with_keep(&mut self, keep: usize) {
        self.stash.push(self.stack.len() - keep);
    }

    pub(crate) fn unstash(&mut self) {
        self.stack.truncate(self.stash.pop().unwrap());
    }

    pub(crate) fn push_var(&mut self, var: Variable) {
        self.stack.push(var)
    }

    pub(crate) fn pop_var_with_type(
        &mut self,
        type_: &ValType,
    ) -> Result<Variable, ValidationError> {
        if self
            .stash
            .last()
            .map_or(false, |&stash| stash >= self.stack.len())
        {
            return Err(ValidationError::Msg("stack underflow".into()));
        }

        let var = match self.stack.pop() {
            Some(var) => var,
            None => return Err(ValidationError::Msg("stack underflow".into())),
        };
        if var.type_ != *type_ {
            return Err(ValidationError::Msg("type mismatch".into()));
        }
        Ok(var)
    }

    pub(crate) fn pop_var(&mut self) -> Result<Variable, ValidationError> {
        if *self.stash.last().unwrap() >= self.stack.len() {
            return Err(ValidationError::Msg("stack underflow".into()));
        }

        self.stack
            .pop()
            .ok_or(ValidationError::Msg("stack underflow".into()))
    }

    pub(crate) fn len(&self) -> usize {
        self.stack.len() - self.stash.last().unwrap()
    }
}

impl Index<usize> for ParserStack {
    type Output = Variable;

    fn index(&self, index: usize) -> &Self::Output {
        &self.stack[self.stash.last().unwrap() + index]
    }
}
