use super::{context::Context, function::Function};
use crate::{util::to_c_str, TranslationError};
use llvm_sys::{
    core::{
        LLVMAddFunction, LLVMAddGlobal, LLVMFunctionType, LLVMGetNamedFunction, LLVMGetNamedGlobal,
        LLVMModuleCreateWithNameInContext, LLVMSetFunctionCallConv, LLVMSetLinkage,
    },
    prelude::{LLVMModuleRef, LLVMTypeRef, LLVMValueRef},
    LLVMCallConv, LLVMLinkage,
};

pub struct Module {
    inner: LLVMModuleRef,
}

impl Module {
    pub(crate) fn new(name: &str, context: &Context) -> Self {
        Self {
            inner: unsafe {
                LLVMModuleCreateWithNameInContext(to_c_str(name).as_ptr(), context.get())
            },
        }
    }

    pub(crate) fn get(&self) -> LLVMModuleRef {
        self.inner
    }

    pub(crate) fn find_func(&self, name: &str, ty: LLVMTypeRef) -> Option<Function> {
        unsafe {
            Function::new(
                LLVMGetNamedFunction(self.get(), to_c_str(name).as_ptr()),
                ty,
            )
        }
    }

    pub(crate) fn add_function(
        &self,
        name: &str,
        ty: LLVMTypeRef,
        linkage: LLVMLinkage,
        call_conv: LLVMCallConv,
    ) -> Function {
        let fn_val = unsafe { LLVMAddFunction(self.get(), to_c_str(name).as_ptr(), ty) };
        unsafe { LLVMSetLinkage(fn_val, linkage) }
        unsafe { LLVMSetFunctionCallConv(fn_val, call_conv as u32) };
        Function::new(fn_val, ty).unwrap()
    }

    pub fn create_func_type(
        return_type: LLVMTypeRef,
        param_types: &mut [LLVMTypeRef],
    ) -> LLVMTypeRef {
        unsafe {
            LLVMFunctionType(
                return_type,
                param_types.as_mut_ptr(),
                param_types.len() as u32,
                false.into(),
            )
        }
    }

    /// Get an intrinsic function by name. Discriminating types are used to select the correct overload. Only supply parameter types required for overload selection.
    pub(crate) fn get_intrinsic_func(
        &self,
        name: &str,
        param_types: &mut [LLVMTypeRef],
        ret_type: LLVMTypeRef,
    ) -> Result<Function, TranslationError> {
        let function_type = unsafe {
            LLVMFunctionType(
                ret_type,
                param_types.as_mut_ptr(),
                param_types.len() as u32,
                false.into(),
            )
        };
        if let Some(f) = self.find_func(name, function_type) {
            return Ok(f);
        }

        Ok(self.add_function(
            name,
            function_type,
            LLVMLinkage::LLVMExternalLinkage,
            LLVMCallConv::LLVMFastCallConv,
        ))
    }

    #[cfg(debug_assertions)]
    pub(crate) fn print_to_file(&self) {
        use std::ptr::null_mut;
        unsafe {
            llvm_sys::core::LLVMPrintModuleToFile(
                self.get(),
                to_c_str("debug_output.ll").as_ptr(),
                null_mut(),
            )
        };
    }

    #[allow(dead_code)]
    #[cfg(debug_assertions)]
    pub(crate) fn print_to_string(&self) -> String {
        use llvm_sys::core::LLVMDisposeMessage;
        use std::ffi::CStr;

        let s_ptr = unsafe { llvm_sys::core::LLVMPrintModuleToString(self.inner) };
        let r_string = unsafe { CStr::from_ptr(s_ptr) }
            .to_str()
            .unwrap()
            .to_string();
        unsafe { LLVMDisposeMessage(s_ptr) }
        r_string
    }

    pub(crate) fn get_global(&self, name: &str) -> Result<LLVMValueRef, TranslationError> {
        let global_addr = unsafe { LLVMGetNamedGlobal(self.get(), to_c_str(name).as_ptr()) };
        if global_addr.is_null() {
            Err(TranslationError::Msg(format!("Global {} not found.", name)))
        } else {
            Ok(global_addr)
        }
    }

    pub(crate) fn add_global(&self, name: &str, ty: LLVMTypeRef) {
        unsafe { LLVMAddGlobal(self.get(), ty, to_c_str(name).as_ptr()) };
    }
}

impl Drop for Module {
    fn drop(&mut self) {
        // modules are owned by execution engines which also free them.
    }
}
