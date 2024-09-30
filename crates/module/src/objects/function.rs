use crate::{basic_block::BasicBlock, ModuleMetadata};
use rkyv::{Archive, Deserialize, Serialize};
use std::vec::Vec;
use wasm_types::{FuncIdx, TypeIdx, ValType};

#[derive(Debug, Default, Clone, Deserialize, Serialize, Archive)]
pub struct Function {
    pub type_idx: u32,
    pub source_ir: Option<FunctionIR>,
    pub source_import: Option<FunctionImport>,
    pub source_unparsed: Option<FunctionUnparsed>,
    pub source_llvm: Option<FunctionLLVM>,
}

impl Function {
    pub fn query_function_name(func_idx: FuncIdx, module: &ModuleMetadata) -> Option<&str> {
        module.exports.find_function_name(func_idx)
    }

    pub fn debug_function_name(func_idx: FuncIdx, module: &ModuleMetadata) -> String {
        Self::query_function_name(func_idx, module)
            .map(|s| s.to_string())
            .unwrap_or(format!("<anonymous:{func_idx}>"))
    }

    pub fn new(type_idx: TypeIdx) -> Self {
        Self {
            type_idx,
            ..Default::default()
        }
    }

    #[inline]
    pub fn get_ir(&self) -> Option<&FunctionIR> {
        self.source_ir.as_ref()
    }

    #[inline]
    pub fn add_ir(&mut self, ir: FunctionIR) {
        self.source_ir = Some(ir);
    }

    #[inline]
    pub fn get_import(&self) -> Option<&FunctionImport> {
        self.source_import.as_ref()
    }

    #[inline]
    pub fn add_import(&mut self, import: FunctionImport) {
        self.source_import = Some(import);
    }

    #[inline]
    pub fn get_unparsed_mem(&self) -> Option<FunctionUnparsed> {
        self.source_unparsed.clone()
    }

    #[inline]
    pub fn add_unparsed_mem(&mut self, offset: usize, length: u32) {
        self.source_unparsed = Some(FunctionUnparsed {
            offset,
            size: length as usize,
        });
    }

    #[inline]
    pub fn add_precompiled_llvm(&mut self, offset: u64, size: usize) {
        self.source_llvm = Some(FunctionLLVM { offset, size });
    }

    #[inline]
    pub fn get_precompiled_llvm(&self) -> Option<FunctionLLVM> {
        self.source_llvm.clone()
    }
}

/// Intermediate representation of the function.
#[derive(Debug, Clone, Default, Deserialize, Serialize, Archive)]
pub struct FunctionIR {
    pub locals: Vec<ValType>,
    pub bbs: Vec<BasicBlock>,
    pub num_vars: usize,
}

/// Function import information.
#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub struct FunctionImport {
    pub import_idx: u32,
}

/// Unparsed binary wasm function code section. (offset into memory region (mmaped for file), length)
#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub struct FunctionUnparsed {
    pub offset: usize,
    pub size: usize,
}

/// Precompiled LLVM function, stored in memory.
///
/// Note: This is used for AOT compiled functions.
///       The stored value is a pointer + size to an LLVM memory buffer of the object file containing the function's symbol.
#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub struct FunctionLLVM {
    pub offset: u64,
    pub size: usize,
}
