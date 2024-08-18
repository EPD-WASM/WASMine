use super::{builder::Builder, module::Module};
use crate::util::c_str;
use llvm_sys::{
    core::{LLVMAppendBasicBlockInContext, LLVMContextCreate, LLVMContextDispose},
    prelude::{LLVMBasicBlockRef, LLVMContextRef, LLVMValueRef},
};
use std::rc::Rc;

pub struct Context {
    inner: LLVMContextRef,
}

impl Context {
    pub fn create() -> Self {
        Self {
            inner: unsafe { LLVMContextCreate() },
        }
    }

    pub(crate) fn get(&self) -> LLVMContextRef {
        self.inner
    }

    pub(crate) fn create_builder(&self, module: Rc<Module>) -> Builder {
        Builder::create(self, module)
    }

    pub(crate) fn append_basic_block(&self, func: LLVMValueRef, name: &str) -> LLVMBasicBlockRef {
        unsafe { LLVMAppendBasicBlockInContext(self.inner, func, c_str(name).as_ptr()) }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            LLVMContextDispose(self.inner);
        }
    }
}
