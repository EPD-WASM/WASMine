use thiserror::Error;

#[derive(Error, Debug)]
pub enum TranslationError {
    #[error("Unimplemented: {0}")]
    Unimplemented(String),
    #[error("ir decoding error: {0}")]
    IRDecodingError(#[from] ir::DecodingError),
    #[error("missing llvm intrinsic")]
    MissingIntrinsic,
    #[error("function not found")]
    FunctionNotFound,
    #[error("generic: {0}")]
    Msg(String),
    #[error("LLVM setup error: {0}")]
    LLVMSetup(String),
    #[error("LLVM jit executor error: {0}")]
    ExecutionError(String),
}

#[derive(Error, Debug)]
pub enum ExecutionError {
    #[error("Translation Error: {0}")]
    TranslationError(#[from] TranslationError),
    #[error("LLVM error: {0}")]
    LLVM(String),
    #[error("{0}")]
    Msg(String),
    #[error("Function not found")]
    FunctionNotFound,
}

impl From<llvm_sys::error::LLVMErrorRef> for ExecutionError {
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn from(err: llvm_sys::error::LLVMErrorRef) -> Self {
        let msg = unsafe {
            let c_str = llvm_sys::error::LLVMGetErrorMessage(err);
            let msg = std::ffi::CStr::from_ptr(c_str)
                .to_string_lossy()
                .into_owned();
            llvm_sys::error::LLVMDisposeErrorMessage(c_str);
            msg
        };
        ExecutionError::LLVM(msg)
    }
}

impl From<*mut i8> for ExecutionError {
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    fn from(err_str: *mut i8) -> Self {
        let msg = unsafe {
            std::ffi::CStr::from_ptr(err_str)
                .to_string_lossy()
                .into_owned()
        };
        ExecutionError::LLVM(msg)
    }
}

impl From<ExecutionError> for TranslationError {
    fn from(err: ExecutionError) -> Self {
        TranslationError::ExecutionError(err.to_string())
    }
}
