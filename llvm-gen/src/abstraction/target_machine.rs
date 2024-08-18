use crate::ExecutionError;
use llvm_sys::target_machine::{
    LLVMCodeGenOptLevel, LLVMCodeModel, LLVMCreateTargetMachine, LLVMGetDefaultTargetTriple,
    LLVMGetFirstTarget, LLVMGetHostCPUFeatures, LLVMGetHostCPUName, LLVMRelocMode,
    LLVMSetTargetMachineFastISel, LLVMTargetMachineRef,
};
use once_cell::sync::Lazy;
use std::ffi::CString;

pub(crate) static TARGET_TRIPLE: Lazy<CString> = Lazy::new(|| unsafe {
    let target_triple = LLVMGetDefaultTargetTriple();
    let target_triple = CString::from_raw(target_triple);
    log::debug!("using LLVM target triple: {:?}", target_triple);
    target_triple
});
static CPU: Lazy<CString> = Lazy::new(|| unsafe {
    let cpu = LLVMGetHostCPUName();
    let cpu = CString::from_raw(cpu);
    log::debug!("LLVM detected CPU: {:?}", cpu);
    cpu
});
static CPU_FEATURES: Lazy<CString> = Lazy::new(|| unsafe {
    let features = LLVMGetHostCPUFeatures();
    CString::from_raw(features)
});
// initialize llvm / target
static LLVM_TARGET_INIT: Lazy<Result<(), ExecutionError>> = Lazy::new(|| {
    if 1 == unsafe { llvm_sys::target::LLVM_InitializeNativeTarget() } {
        return Err(ExecutionError::LLVM(
            "Unknown error in initializing native target".into(),
        ));
    }
    if 1 == unsafe { llvm_sys::target::LLVM_InitializeNativeAsmPrinter() } {
        return Err(ExecutionError::LLVM(
            "Unknown error in initializing native asm printer".into(),
        ));
    }
    Ok(())
});

pub(crate) struct TargetMachine(LLVMTargetMachineRef);

impl TargetMachine {
    pub(crate) fn create_default() -> Result<Self, ExecutionError> {
        LLVM_TARGET_INIT.clone()?;
        let target = unsafe { LLVMGetFirstTarget() };
        let target_machine = unsafe {
            LLVMCreateTargetMachine(
                target,
                TARGET_TRIPLE.as_ptr(),
                CPU.as_ptr(),
                CPU_FEATURES.as_ptr(),
                LLVMCodeGenOptLevel::LLVMCodeGenLevelDefault,
                LLVMRelocMode::LLVMRelocDefault,
                LLVMCodeModel::LLVMCodeModelJITDefault,
            )
        };
        unsafe { LLVMSetTargetMachineFastISel(target_machine, true.into()) };
        Ok(Self(target_machine))
    }

    pub(crate) fn into_raw(self) -> LLVMTargetMachineRef {
        self.0
    }
}
