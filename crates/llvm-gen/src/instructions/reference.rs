use crate::{error::TranslationError, translator::Translator};
use llvm_sys::{prelude::LLVMValueRef, LLVMIntPredicate};
use module::{
    instructions::{
        ReferenceFunctionInstruction, ReferenceIsNullInstruction, ReferenceNullInstruction,
    },
    objects::value::{Reference, Value, ValueRaw},
    InstructionDecoder,
};
use wasm_types::{InstructionType, ReferenceInstructionType};

impl Translator<'_> {
    pub(crate) fn translate_reference(
        &self,
        instr_type: ReferenceInstructionType,
        instruction: InstructionType,
        decoder: &mut InstructionDecoder,
        variable_map: &mut [LLVMValueRef],
    ) -> Result<(), TranslationError> {
        match instr_type {
            ReferenceInstructionType::RefFunc => {
                let instr = decoder.read::<ReferenceFunctionInstruction>(instruction)?;
                variable_map[instr.out1] = self.builder.const_i64(instr.func_idx as u64);
            }
            ReferenceInstructionType::RefIsNull => {
                let instr = decoder.read::<ReferenceIsNullInstruction>(instruction)?;
                let val = variable_map[instr.in1];
                let val = self.builder.build_icmp(
                    LLVMIntPredicate::LLVMIntEQ,
                    val,
                    self.builder
                        .const_i64(ValueRaw::from(Value::Reference(Reference::Null)).as_u64()),
                    "ref_is_null",
                );
                variable_map[instr.out1] =
                    self.builder
                        .build_int_cast(val, self.builder.i32(), false, "ref_is_null")
            }
            ReferenceInstructionType::RefNull => {
                let instr = decoder.read::<ReferenceNullInstruction>(instruction)?;
                variable_map[instr.out1] = self
                    .builder
                    .const_i64(ValueRaw::from(Value::Reference(Reference::Null)).as_u64());
            }
        }
        Ok(())
    }
}
