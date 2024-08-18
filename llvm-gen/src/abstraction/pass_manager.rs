use super::module::Module;
use crate::{abstraction::target_machine::TargetMachine, util::c_str, ExecutionError};
use llvm_sys::transforms::pass_builder::{LLVMCreatePassBuilderOptions, LLVMRunPasses};

pub(crate) struct PassManager;

impl PassManager {
    pub(crate) fn optimize_module(module: &Module) -> Result<(), ExecutionError> {
        let options = unsafe { LLVMCreatePassBuilderOptions() };
        let optimization_passes = "mem2reg,gvn,reassociate,adce,simplifycfg";
        log::debug!("running passes '{optimization_passes}' on translated module");
        let err = unsafe {
            LLVMRunPasses(
                module.get(),
                c_str(optimization_passes).as_ptr(),
                TargetMachine::create_default()?.into_raw(),
                options,
            )
        };
        if !err.is_null() {
            return Err(err.into());
        }
        Ok(())
    }
}
