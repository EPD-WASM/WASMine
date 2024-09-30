use crate::util::c_str;
use crate::{abstraction::function::Function, TranslationError, Translator};
use llvm_sys::core::LLVMBuildAdd;
use llvm_sys::prelude::LLVMValueRef;
use module::instructions::{
    DataDropInstruction, MemoryCopyInstruction, MemoryFillInstruction, MemoryGrowInstruction,
    MemoryInitInstruction, MemorySizeInstruction,
};
use module::objects::memory::MemArg;
use module::{
    instructions::{LoadInstruction, StoreInstruction},
    InstructionDecoder,
};
use wasm_types::{InstructionType, LoadOp, MemoryInstructionCategory, MemoryOp, StoreOp};

impl Translator<'_> {
    pub(crate) fn translate_memory(
        &self,
        instr_type: MemoryInstructionCategory,
        instruction: InstructionType,
        decoder: &mut InstructionDecoder,
        variable_map: &mut [LLVMValueRef],
        llvm_function: &Function,
    ) -> Result<(), TranslationError> {
        match instr_type {
            MemoryInstructionCategory::Load(_) => {
                let instr = decoder.read::<LoadInstruction>(instruction)?;
                self.compile_load(instr, variable_map, llvm_function)
            }
            MemoryInstructionCategory::Store(_) => {
                let instr = decoder.read::<StoreInstruction>(instruction)?;
                self.compile_store(instr, variable_map, llvm_function)
            }
            MemoryInstructionCategory::Memory(op) => match op {
                MemoryOp::Size => {
                    let instr = decoder.read::<MemorySizeInstruction>(instruction)?;
                    let mem_size = self.ec_get_mem_size(Self::get_rt_ref(llvm_function), 0);
                    variable_map[instr.out1] = mem_size;
                    Ok(())
                }
                MemoryOp::Grow => {
                    let instr = decoder.read::<MemoryGrowInstruction>(instruction)?;
                    let grow_by = variable_map[instr.in1];
                    let res = self.memory_grow(Self::get_rt_ref(llvm_function), 0, grow_by);
                    variable_map[instr.out1] = res;
                    Ok(())
                }
                MemoryOp::Init => {
                    let instr = decoder.read::<MemoryInitInstruction>(instruction)?;
                    self.memory_init(
                        Self::get_rt_ref(llvm_function),
                        0,
                        instr.data_idx,
                        variable_map[instr.s],
                        variable_map[instr.d],
                        variable_map[instr.n],
                    );
                    Ok(())
                }
                MemoryOp::Fill => {
                    let instr = decoder.read::<MemoryFillInstruction>(instruction)?;
                    let value = self.builder.build_int_cast(
                        variable_map[instr.val],
                        self.builder.i8(),
                        false,
                        "cast_fill_val",
                    );
                    self.memory_fill(
                        Self::get_rt_ref(llvm_function),
                        0,
                        variable_map[instr.d],
                        variable_map[instr.n],
                        value,
                    );
                    Ok(())
                }
                MemoryOp::Copy => {
                    let instr = decoder.read::<MemoryCopyInstruction>(instruction)?;
                    self.memory_copy(
                        Self::get_rt_ref(llvm_function),
                        0,
                        variable_map[instr.s],
                        variable_map[instr.d],
                        variable_map[instr.n],
                    );
                    Ok(())
                }
                MemoryOp::Drop => {
                    let instr = decoder.read::<DataDropInstruction>(instruction)?;
                    self.data_drop(Self::get_rt_ref(llvm_function), instr.data_idx);
                    Ok(())
                }
            },
        }
    }

    pub(crate) fn compile_load(
        &self,
        instr: LoadInstruction,
        variable_map: &mut [LLVMValueRef],
        llvm_function: &Function,
    ) -> Result<(), TranslationError> {
        let memory_ptr = self.ec_get_mem_ptr(Self::get_rt_ref(llvm_function), 0);
        let addr = self.calc_addr(variable_map[instr.addr], memory_ptr, &instr.memarg);
        let out_ty = self
            .builder
            .valtype2llvm(wasm_types::ValType::Number(instr.out1_type));
        variable_map[instr.out1] = match instr.operation {
            LoadOp::INNLoad | LoadOp::FNNLoad => self.builder.build_load(out_ty, addr, "load"),
            LoadOp::INNLoad8S => {
                let val = self.builder.build_load(self.builder.i8(), addr, "load");
                self.builder.build_int_cast(val, out_ty, true, "upcast")
            }
            LoadOp::INNLoad8U => {
                let val = self.builder.build_load(self.builder.i8(), addr, "load");
                self.builder.build_int_cast(val, out_ty, false, "upcast")
            }
            LoadOp::INNLoad16S => {
                let val = self.builder.build_load(self.builder.i16(), addr, "load");
                self.builder.build_int_cast(val, out_ty, true, "upcast")
            }
            LoadOp::INNLoad16U => {
                let val = self.builder.build_load(self.builder.i16(), addr, "load");
                self.builder.build_int_cast(val, out_ty, false, "upcast")
            }
            LoadOp::INNLoad32S => {
                let val = self.builder.build_load(self.builder.i32(), addr, "load");
                self.builder.build_int_cast(val, out_ty, true, "upcast")
            }
            LoadOp::INNLoad32U => {
                let val = self.builder.build_load(self.builder.i32(), addr, "load");
                self.builder.build_int_cast(val, out_ty, false, "upcast")
            }
        };
        Ok(())
    }

    fn calc_addr(
        &self,
        mut base_addr: LLVMValueRef,
        memory_ptr: LLVMValueRef,
        memarg: &MemArg,
    ) -> LLVMValueRef {
        if memarg.offset != 0 {
            base_addr = unsafe {
                LLVMBuildAdd(
                    self.builder.get(),
                    base_addr,
                    self.builder.const_i32(memarg.offset),
                    c_str("add_memarg_offset").as_ptr(),
                )
            };
        }
        // memarg.align may be ignored (see reference)
        self.builder.build_gep(
            self.builder.i8(),
            memory_ptr,
            &mut [base_addr],
            "get_storage_pos",
        )
    }

    pub(crate) fn compile_store(
        &self,
        instr: StoreInstruction,
        variable_map: &mut [LLVMValueRef],
        llvm_function: &Function,
    ) -> Result<(), TranslationError> {
        let memory_ptr = self.ec_get_mem_ptr(Self::get_rt_ref(llvm_function), 0);
        let val = variable_map[instr.value_in];
        let addr = self.calc_addr(variable_map[instr.addr_in], memory_ptr, &instr.memarg);

        let val = match instr.operation {
            StoreOp::INNStore | StoreOp::FNNStore => val,
            StoreOp::INNStore8 => {
                self.builder
                    .build_int_cast(val, self.builder.i8(), false, "downcast")
            }
            StoreOp::INNStore16 => {
                self.builder
                    .build_int_cast(val, self.builder.i16(), false, "downcast")
            }
            StoreOp::INNStore32 => {
                self.builder
                    .build_int_cast(val, self.builder.i32(), false, "downcast")
            }
        };
        self.builder.build_store(val, addr);
        Ok(())
    }
}
