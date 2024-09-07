use crate::{TranslationError, Translator};
use ir::{instructions::SelectInstruction, InstructionDecoder};
use llvm_sys::prelude::LLVMValueRef;
use wasm_types::{InstructionType, ParametricInstructionType, ValType};

impl Translator {
    pub(crate) fn translate_parametric(
        &self,
        instr_type: ParametricInstructionType,
        instruction: InstructionType,
        decoder: &mut InstructionDecoder,
        variable_map: &mut [LLVMValueRef],
    ) -> Result<(), TranslationError> {
        match instr_type {
            ParametricInstructionType::Select => {
                let instr = decoder.read::<SelectInstruction>(instruction)?;
                let select_val = variable_map[instr.select_val as usize];
                let then_val = variable_map[instr.input_vals[0] as usize];
                let else_val = variable_map[instr.input_vals[1] as usize];

                let select_val = self.builder.build_icmp(
                    llvm_sys::LLVMIntPredicate::LLVMIntNE,
                    select_val,
                    self.builder.const_zero(ValType::i32()),
                    "ToBool",
                );
                variable_map[instr.out1 as usize] = self
                    .builder
                    .build_select(select_val, then_val, else_val, "select");
            }
            ParametricInstructionType::Drop => {
                unreachable!("Drop instructions are not emitted by the parser.")
            }
        }
        Ok(())
    }
}
