use std::ops::Range;

use super::{
    element::Element,
    value::{ExternReference, FunctionReference},
};
use crate::structs::value::Reference;
use thiserror::Error;
use wasm_types::{Limits, RefType, TableType};

#[derive(Debug, Clone)]
enum TableData {
    FunctionReference(Vec<FunctionReference>),
    ExternReference(Vec<ExternReference>),
}

impl TableData {
    fn len(&self) -> usize {
        match self {
            TableData::FunctionReference(v) => v.len(),
            TableData::ExternReference(v) => v.len(),
        }
    }
}

// this could use some refactoring, tabletype contains the reference type but so does the struct itself
#[derive(Debug, Clone)]
pub struct Table {
    pub limits: Limits,
    data: TableData,
}

#[derive(Debug, Clone, Error)]
pub enum TableError {
    #[error("Table index out of range: {index}, limits: {limits:?}")]
    OutOfRangeError { index: usize, limits: Limits },
    #[error("Table type mismatch in operation")]
    TypeMismatchError,
}

pub type TableSize = u32;

pub trait Tablelike {
    fn get(&self, index: u32) -> Option<Reference>;
    fn set(&mut self, index: u32, data: Reference) -> Result<(), TableError>;
    fn size(&self) -> TableSize;
    fn grow(&mut self) -> Result<TableSize, TableError>;
    fn fill(&mut self, range: Range<usize>, data: Reference) -> Result<(), TableError>;
    fn copy_within(&mut self, src: Range<usize>, dst: u32) -> Result<(), TableError>;
    fn copy_inter(
        src_table: &mut Table,
        src_range: Range<u32>,
        dst_table: &mut Table,
        dst_range: Range<u32>,
    ) -> Result<(), TableError>;
    fn init(&mut self, offset: u32, data: Vec<Element>) -> Result<(), TableError>;
    fn get_ref_type(&self) -> RefType;
}

impl Table {
    pub fn new(type_: TableType) -> Self {
        let data = match type_.ref_type {
            RefType::FunctionReference => TableData::FunctionReference(Vec::new()),
            RefType::ExternReference => TableData::ExternReference(Vec::new()),
        };
        Self {
            limits: type_.lim,
            data,
        }
    }

    fn len(&self) -> usize {
        self.data.len()
    }
}

impl Tablelike for Table {
    fn get(&self, offset: u32) -> Option<Reference> {
        let i = offset as usize;
        match &self.data {
            TableData::FunctionReference(fn_refs) => {
                fn_refs.get(i).cloned().map(Reference::Function)
            }
            TableData::ExternReference(ext_refs) => ext_refs.get(i).cloned().map(Reference::Extern),
        }
    }

    fn set(&mut self, offset: u32, data: Reference) -> Result<(), TableError> {
        if offset as usize >= self.data.len() {
            return Err(TableError::OutOfRangeError {
                index: offset as usize,
                limits: self.limits.clone(),
            });
        }

        match (&mut self.data, data) {
            (TableData::FunctionReference(ref mut v), Reference::Function(r)) => {
                v[offset as usize] = r;
            }
            (TableData::ExternReference(ref mut v), Reference::Extern(r)) => {
                v[offset as usize] = r;
            }
            _ => return Err(TableError::TypeMismatchError),
        };

        Ok(())
    }

    fn size(&self) -> TableSize {
        self.data.len() as TableSize
    }

    fn grow(&mut self) -> Result<TableSize, TableError> {
        todo!()
    }

    fn fill(&mut self, range: Range<usize>, data: Reference) -> Result<(), TableError> {
        if range.end > self.data.len() {
            return Err(TableError::OutOfRangeError {
                index: range.end,
                limits: self.limits.clone(),
            });
        }

        match (&mut self.data, data) {
            (TableData::FunctionReference(ref mut v), Reference::Function(r)) => {
                v[range].iter_mut().for_each(|x| *x = r.clone());
            }
            (TableData::ExternReference(ref mut v), Reference::Extern(r)) => {
                v[range].iter_mut().for_each(|x| *x = r.clone());
            }
            _ => return Err(TableError::TypeMismatchError),
        };

        Ok(())
    }

    fn copy_within(&mut self, src: Range<usize>, dst: u32) -> Result<(), TableError> {
        if src.end > self.data.len() || dst as usize + src.len() > self.data.len() {
            return Err(TableError::OutOfRangeError {
                index: src.end,
                limits: self.limits.clone(),
            });
        }

        Ok(())
    }

    fn copy_inter(
        _src_table: &mut Table,
        _src_range: Range<u32>,
        _dst_table: &mut Table,
        _dst_range: Range<u32>,
    ) -> Result<(), TableError> {
        todo!()
    }

    fn init(&mut self, _offset: u32, _data: Vec<Element>) -> Result<(), TableError> {
        todo!()
    }

    fn get_ref_type(&self) -> RefType {
        match self.data {
            TableData::FunctionReference(_) => RefType::FunctionReference,
            TableData::ExternReference(_) => RefType::ExternReference,
        }
    }
}
