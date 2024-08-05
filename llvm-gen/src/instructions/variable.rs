use crate::{TranslationError, Translator};
use ir::{
    instructions::{
        GlobalGetInstruction, GlobalSetInstruction, LocalGetInstruction, LocalSetInstruction,
        LocalTeeInstruction,
    },
    InstructionDecoder,
};
use llvm_sys::prelude::{LLVMTypeRef, LLVMValueRef};
use wasm_types::{GlobalType, InstructionType, VariableInstructionType};

impl Translator {
    pub(crate) fn translate_variable(
        &self,
        instr_type: VariableInstructionType,
        instruction: InstructionType,
        decoder: &mut InstructionDecoder,
        variable_map: &mut [LLVMValueRef],
        local_map: &[(LLVMValueRef, LLVMTypeRef)],
    ) -> Result<(), TranslationError> {
        match instr_type {
            VariableInstructionType::LocalGet => {
                let instr = decoder.read::<LocalGetInstruction>(instruction)?;
                let (local_val, local_ty) = local_map[instr.local_idx as usize];
                let value = self.builder.build_load(local_ty, local_val, "local_get");
                variable_map[instr.out1 as usize] = value;
            }
            VariableInstructionType::LocalSet => {
                let instr = decoder.read::<LocalSetInstruction>(instruction)?;
                let (local_val, _) = local_map[instr.local_idx as usize];
                let value = variable_map[instr.in1 as usize];
                self.builder.build_store(value, local_val);
            }
            VariableInstructionType::LocalTee => {
                let instr = decoder.read::<LocalTeeInstruction>(instruction)?;
                let (local_val, _) = local_map[instr.local_idx as usize];
                let value = variable_map[instr.in1 as usize];
                self.builder.build_store(value, local_val);
                variable_map[instr.out1 as usize] = value;
            }
            VariableInstructionType::GlobalGet => {
                let instr = decoder.read::<GlobalGetInstruction>(instruction)?;
                let wasm_global = &self.wasm_module.globals[instr.global_idx as usize];
                let global_type = match wasm_global.r#type {
                    GlobalType::Const(ty) => ty,
                    GlobalType::Mut(ty) => ty,
                };
                let global_name = format!("__wasmine_global__{}", instr.global_idx);
                let global = self.module.get_global(&global_name)?;
                variable_map[instr.out1 as usize] = self.builder.build_load(
                    self.builder.valtype2llvm(global_type),
                    global,
                    "global_get",
                )
            }
            VariableInstructionType::GlobalSet => {
                let instr = decoder.read::<GlobalSetInstruction>(instruction)?;
                let global_name = format!("__wasmine_global__{}", instr.global_idx);
                let global = self.module.get_global(&global_name)?;
                self.builder
                    .build_store(variable_map[instr.in1 as usize], global)
            }
        }
        Ok(())
    }
}
