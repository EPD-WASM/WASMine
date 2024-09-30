use crate::{abstraction::function::Function, TranslationError, Translator};
use llvm_sys::prelude::LLVMValueRef;
use module::{
    instructions::{
        ElemDropInstruction, TableCopyInstruction, TableFillInstruction, TableGetInstruction,
        TableGrowInstruction, TableInitInstruction, TableSetInstruction, TableSizeInstruction,
    },
    InstructionDecoder,
};
use wasm_types::{InstructionType, TableInstructionCategory, ValType};

impl Translator<'_> {
    pub(crate) fn translate_table(
        &self,
        instr_type: TableInstructionCategory,
        instruction: InstructionType,
        decoder: &mut InstructionDecoder,
        variable_map: &mut [LLVMValueRef],
        llvm_function: &Function,
    ) -> Result<(), TranslationError> {
        match instr_type {
            TableInstructionCategory::Init => {
                let instr = decoder.read::<TableInitInstruction>(instruction)?;
                let n = variable_map[instr.n];
                let d = variable_map[instr.d];
                let s = variable_map[instr.s];
                self.table_init(
                    Self::get_rt_ref(llvm_function),
                    instr.table_idx,
                    instr.elem_idx,
                    s,
                    d,
                    n,
                );
            }
            TableInstructionCategory::Size => {
                let instr = decoder.read::<TableSizeInstruction>(instruction)?;
                variable_map[instr.out1] =
                    self.table_size(Self::get_rt_ref(llvm_function), instr.table_idx);
            }
            TableInstructionCategory::Copy => {
                let instr = decoder.read::<TableCopyInstruction>(instruction)?;
                self.table_copy(
                    Self::get_rt_ref(llvm_function),
                    instr.table_idx_y,
                    instr.table_idx_x,
                    variable_map[instr.s],
                    variable_map[instr.d],
                    variable_map[instr.n],
                )
            }
            TableInstructionCategory::Fill => {
                let instr = decoder.read::<TableFillInstruction>(instruction)?;
                self.table_fill(
                    Self::get_rt_ref(llvm_function),
                    instr.table_idx,
                    variable_map[instr.i],
                    variable_map[instr.n],
                    variable_map[instr.ref_value],
                )
            }
            TableInstructionCategory::Drop => {
                let instr = decoder.read::<ElemDropInstruction>(instruction)?;
                self.elem_drop(Self::get_rt_ref(llvm_function), instr.elem_idx);
            }
            TableInstructionCategory::Get => {
                let instr = decoder.read::<TableGetInstruction>(instruction)?;
                variable_map[instr.out1] = self.table_get(
                    Self::get_rt_ref(llvm_function),
                    instr.table_idx,
                    variable_map[instr.idx],
                    self.builder.valtype2llvm(ValType::Reference(
                        self.wasm_module_meta.tables[instr.table_idx as usize]
                            .r#type
                            .ref_type,
                    )),
                );
            }
            TableInstructionCategory::Set => {
                let instr = decoder.read::<TableSetInstruction>(instruction)?;
                self.table_set(
                    Self::get_rt_ref(llvm_function),
                    instr.table_idx,
                    variable_map[instr.in1],
                    variable_map[instr.idx],
                );
            }
            TableInstructionCategory::Grow => {
                let instr = decoder.read::<TableGrowInstruction>(instruction)?;
                variable_map[instr.out1] = self.table_grow(
                    Self::get_rt_ref(llvm_function),
                    instr.table_idx,
                    variable_map[instr.size],
                    variable_map[instr.value_to_fill],
                );
            }
        }
        Ok(())
    }
}
