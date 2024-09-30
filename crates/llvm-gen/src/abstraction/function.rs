use llvm_sys::{
    core::LLVMGetParam,
    prelude::{LLVMTypeRef, LLVMValueRef},
};

#[derive(Clone, Copy)]
pub(crate) struct Function {
    inner: LLVMValueRef,
    ty: LLVMTypeRef,
}

impl Function {
    pub(crate) fn new(inner: LLVMValueRef, ty: LLVMTypeRef) -> Option<Self> {
        if inner.is_null() {
            None
        } else {
            Some(Self { inner, ty })
        }
    }

    pub(crate) fn get(&self) -> LLVMValueRef {
        self.inner
    }

    pub(crate) fn get_param(&self, index: usize) -> LLVMValueRef {
        unsafe { LLVMGetParam(self.inner, index as u32) }
    }

    pub(crate) fn r#type(&self) -> LLVMTypeRef {
        self.ty
    }
}

#[cfg(debug_assertions)]
mod debug_helper {
    use super::*;
    use llvm_sys::core::{LLVMDisposeMessage, LLVMPrintValueToString};
    use std::ffi::CStr;

    impl Function {
        #[allow(dead_code)]
        #[cfg(debug_assertions)]
        pub(crate) fn print_to_string(&self) -> String {
            let s_ptr = unsafe { LLVMPrintValueToString(self.inner) };
            let r_string = unsafe { CStr::from_ptr(s_ptr) }
                .to_str()
                .unwrap()
                .to_string();
            unsafe { LLVMDisposeMessage(s_ptr) }
            r_string
        }
    }
}
