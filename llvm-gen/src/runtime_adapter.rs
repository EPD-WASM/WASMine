use crate::{
    abstraction::{function::Function, module::Module},
    Translator,
};
use llvm_sys::{
    core::LLVMIntTypeInContext,
    prelude::{LLVMTypeRef, LLVMValueRef},
};
use runtime_interface::{ExecutionContext, MemoryInstance};
use wasm_types::{DataIdx, ElemIdx, MemIdx, TableIdx, ValType};

impl Translator {
    pub(crate) fn ec_memories_ptr(&self, ec_ptr: LLVMValueRef) -> LLVMValueRef {
        let ptr = self.builder.build_gep(
            self.builder.i8(),
            ec_ptr,
            &mut [self
                .builder
                .const_i32(std::mem::offset_of!(ExecutionContext, memories_ptr) as u32)],
            "access_ec_memories_ptr",
        );
        let ptr = self
            .builder
            .build_bitcast(ptr, self.builder.ptr(), "ec_memories_ptr_bitcast");
        self.builder
            .build_load(self.builder.ptr(), ptr, "load_ec_memories_ptr")
    }

    pub(crate) fn ec_get_memory_instance_ptr(
        &self,
        ec_ptr: LLVMValueRef,
        idx: usize,
    ) -> LLVMValueRef {
        let memory_instance_ptr = self.builder.build_gep(
            // size of one array element
            unsafe {
                LLVMIntTypeInContext(
                    self.context.get(),
                    std::mem::size_of::<MemoryInstance>() as u32 * 8,
                )
            },
            self.ec_memories_ptr(ec_ptr),
            &mut [self.builder.const_i32(idx as u32)],
            "access_memory_instance",
        );
        self.builder.build_bitcast(
            memory_instance_ptr,
            self.builder.ptr(),
            "cast_memory_instance_ptr",
        )
    }

    pub(crate) fn ec_get_mem_ptr(&self, ec_ptr: LLVMValueRef, idx: usize) -> LLVMValueRef {
        let mem_instance = self.ec_get_memory_instance_ptr(ec_ptr, idx);
        let mem_ptr_ptr = self.builder.build_gep(
            self.builder.i8(),
            mem_instance,
            &mut [self
                .builder
                .const_i32(std::mem::offset_of!(MemoryInstance, data) as u32)],
            "access_mem_ptr",
        );
        self.builder
            .build_load(self.builder.ptr(), mem_ptr_ptr, "load_mem_ptr")
    }

    pub(crate) fn ec_get_mem_size(&self, ec_ptr: LLVMValueRef, idx: usize) -> LLVMValueRef {
        let mem_instance = self.ec_get_memory_instance_ptr(ec_ptr, idx);
        let mem_size_ptr = self.builder.build_gep(
            self.builder.i8(),
            mem_instance,
            &mut [self
                .builder
                .const_i32(std::mem::offset_of!(MemoryInstance, size) as u32)],
            "access_mem_ptr",
        );
        let mem_size_ptr =
            self.builder
                .build_bitcast(mem_size_ptr, self.builder.ptr(), "mem_size_ptr");
        self.builder
            .build_load(self.builder.i32(), mem_size_ptr, "load_mem_size")
    }

    fn get_rt_func(&self, name: &str, func_type: LLVMTypeRef) -> Function {
        self.module.find_func(name, func_type).unwrap_or_else(|| {
            self.module.add_function(
                name,
                func_type,
                llvm_sys::LLVMLinkage::LLVMExternalLinkage,
                llvm_sys::LLVMCallConv::LLVMCCallConv,
            )
        })
    }

    pub(crate) fn memory_grow(
        &self,
        ctxt: LLVMValueRef,
        memory_idx: usize,
        grow_by: LLVMValueRef,
    ) -> LLVMValueRef {
        let func_type = Module::create_func_type(
            self.builder.i32(),
            &mut [self.builder.ptr(), self.builder.i32(), self.builder.i32()],
        );
        let memory_grow_fn = self.get_rt_func("__wasmine_runtime.memory_grow", func_type);
        self.builder.build_call(
            &memory_grow_fn,
            &mut [ctxt, self.builder.const_i32(memory_idx as u32), grow_by],
            "memory_grow_res",
        )
    }

    pub(crate) fn memory_fill(
        &self,
        ctxt: LLVMValueRef,
        memory_idx: MemIdx,
        offset: LLVMValueRef,
        size: LLVMValueRef,
        value: LLVMValueRef,
    ) {
        let func_type = Module::create_func_type(
            self.builder.void(),
            &mut [
                self.builder.ptr(),
                self.builder.i32(),
                self.builder.i32(),
                self.builder.i32(),
                self.builder.i8(),
            ],
        );
        let memory_fill_fn = self.get_rt_func("__wasmine_runtime.memory_fill", func_type);
        self.builder.build_call(
            &memory_fill_fn,
            &mut [
                ctxt,
                self.builder.const_i32(memory_idx),
                offset,
                size,
                value,
            ],
            "", /* void */
        );
    }

    pub(crate) fn memory_copy(
        &self,
        ctxt: LLVMValueRef,
        memory_idx: MemIdx,
        src_offset: LLVMValueRef,
        dst_offset: LLVMValueRef,
        size: LLVMValueRef,
    ) {
        let func_type = Module::create_func_type(
            self.builder.void(),
            &mut [
                self.builder.ptr(),
                self.builder.i32(),
                self.builder.i32(),
                self.builder.i32(),
                self.builder.i32(),
            ],
        );
        let memory_copy_fn = self.get_rt_func("__wasmine_runtime.memory_copy", func_type);
        self.builder.build_call(
            &memory_copy_fn,
            &mut [
                ctxt,
                self.builder.const_i32(memory_idx),
                src_offset,
                dst_offset,
                size,
            ],
            "", /* void */
        );
    }

    pub(crate) fn data_drop(&self, ctxt: LLVMValueRef, data_idx: DataIdx) {
        let func_type = Module::create_func_type(
            self.builder.void(),
            &mut [self.builder.ptr(), self.builder.i32()],
        );
        let data_drop_fn = self.get_rt_func("__wasmine_runtime.data_drop", func_type);
        self.builder.build_call(
            &data_drop_fn,
            &mut [ctxt, self.builder.const_i32(data_idx)],
            "", /* void */
        );
    }

    pub(crate) fn memory_init(
        &self,
        ctxt: LLVMValueRef,
        memory_idx: u32,
        data_idx: u32,
        src_offset: LLVMValueRef,
        dst_offset: LLVMValueRef,
        size: LLVMValueRef,
    ) {
        let func_type = Module::create_func_type(
            self.builder.void(),
            &mut [
                self.builder.ptr(),
                self.builder.i32(),
                self.builder.i32(),
                self.builder.i32(),
                self.builder.i32(),
                self.builder.i32(),
            ],
        );
        let memory_init_fn = self.get_rt_func("__wasmine_runtime.memory_init", func_type);
        self.builder.build_call(
            &memory_init_fn,
            &mut [
                ctxt,
                self.builder.const_i32(memory_idx),
                self.builder.const_i32(data_idx),
                src_offset,
                dst_offset,
                size,
            ],
            "", /* void */
        );
    }

    pub(crate) fn table_init(
        &self,
        ctxt: LLVMValueRef,
        table_idx: TableIdx,
        elem_idx: ElemIdx,
        src_offset: LLVMValueRef,
        dst_offset: LLVMValueRef,
        len: LLVMValueRef,
    ) {
        let func_type = Module::create_func_type(
            self.builder.void(),
            &mut [
                self.builder.ptr(),
                self.builder.i32(),
                self.builder.i32(),
                self.builder.i32(),
                self.builder.i32(),
                self.builder.i32(),
            ],
        );
        let table_init_fn = self.get_rt_func("__wasmine_runtime.table_init", func_type);
        self.builder.build_call(
            &table_init_fn,
            &mut [
                ctxt,
                self.builder.const_i32(table_idx),
                self.builder.const_i32(elem_idx),
                src_offset,
                dst_offset,
                len,
            ],
            "", /* void */
        );
    }

    pub(crate) fn indirect_call(
        &self,
        ctxt: LLVMValueRef,
        table_idx: u32,
        type_idx: u32,
        entry_idx: LLVMValueRef,
    ) -> LLVMValueRef {
        let func_type = Module::create_func_type(
            self.builder.ptr(),
            &mut [
                self.builder.ptr(),
                self.builder.i32(),
                self.builder.i32(),
                self.builder.i32(),
            ],
        );
        let indirect_call_fn = self.get_rt_func("__wasmine_runtime.indirect_call", func_type);
        self.builder.build_call(
            &indirect_call_fn,
            &mut [
                ctxt,
                self.builder.const_i32(table_idx),
                self.builder.const_i32(type_idx),
                entry_idx,
            ],
            "indirect_call_res",
        )
    }

    pub(crate) fn table_size(&self, ctxt: LLVMValueRef, table_idx: TableIdx) -> LLVMValueRef {
        let func_type = Module::create_func_type(
            self.builder.i32(),
            &mut [self.builder.ptr(), self.builder.i32()],
        );
        let table_size_fn = self.get_rt_func("__wasmine_runtime.table_size", func_type);
        self.builder.build_call(
            &table_size_fn,
            &mut [ctxt, self.builder.const_i32(table_idx)],
            "table_size_res",
        )
    }

    pub(crate) fn table_copy(
        &self,
        ctxt: LLVMValueRef,
        src_table_idx: u32,
        dst_table_idx: u32,
        src_start: LLVMValueRef,
        dst_start: LLVMValueRef,
        len: LLVMValueRef,
    ) {
        let func_type = Module::create_func_type(
            self.builder.void(),
            &mut [
                self.builder.ptr(),
                self.builder.i32(),
                self.builder.i32(),
                self.builder.i32(),
                self.builder.i32(),
                self.builder.i32(),
            ],
        );
        let table_copy_fn = self.get_rt_func("__wasmine_runtime.table_copy", func_type);
        self.builder.build_call(
            &table_copy_fn,
            &mut [
                ctxt,
                self.builder.const_i32(src_table_idx),
                self.builder.const_i32(dst_table_idx),
                src_start,
                dst_start,
                len,
            ],
            "", /* void */
        );
    }

    pub(crate) fn table_fill(
        &self,
        ctxt: LLVMValueRef,
        table_idx: u32,
        start: LLVMValueRef,
        len: LLVMValueRef,
        value: LLVMValueRef,
    ) {
        let func_type = Module::create_func_type(
            self.builder.void(),
            &mut [
                self.builder.ptr(),
                self.builder.i32(),
                self.builder.i32(),
                self.builder.i32(),
                self.builder.valtype2llvm(ValType::funcref()),
            ],
        );
        let table_fill_fn = self.get_rt_func("__wasmine_runtime.table_fill", func_type);
        self.builder.build_call(
            &table_fill_fn,
            &mut [ctxt, self.builder.const_i32(table_idx), start, len, value],
            "", /* void */
        );
    }

    pub(crate) fn table_get(
        &self,
        ctxt: LLVMValueRef,
        table_idx: u32,
        idx: LLVMValueRef,
        _: LLVMTypeRef,
    ) -> LLVMValueRef {
        let func_type = Module::create_func_type(
            self.builder.valtype2llvm(ValType::funcref()),
            &mut [self.builder.ptr(), self.builder.i32(), self.builder.i32()],
        );
        let table_get_fn = self.get_rt_func("__wasmine_runtime.table_get", func_type);
        self.builder.build_call(
            &table_get_fn,
            &mut [ctxt, self.builder.const_i32(table_idx), idx],
            "table_get_res",
        )
    }

    pub(crate) fn table_set(
        &self,
        ctxt: LLVMValueRef,
        table_idx: u32,
        value: LLVMValueRef,
        idx: LLVMValueRef,
    ) {
        let func_type = Module::create_func_type(
            self.builder.void(),
            &mut [
                self.builder.ptr(),
                self.builder.i32(),
                self.builder.valtype2llvm(ValType::funcref()),
                self.builder.i32(),
            ],
        );
        let table_set_fn = self.get_rt_func("__wasmine_runtime.table_set", func_type);
        self.builder.build_call(
            &table_set_fn,
            &mut [ctxt, self.builder.const_i32(table_idx), value, idx],
            "", /* void */
        );
    }

    pub(crate) fn elem_drop(&self, ctxt: LLVMValueRef, elem_idx: u32) {
        let func_type = Module::create_func_type(
            self.builder.void(),
            &mut [self.builder.ptr(), self.builder.i32()],
        );
        let elem_drop_fn = self.get_rt_func("elem_drop", func_type);
        self.builder.build_call(
            &elem_drop_fn,
            &mut [ctxt, self.builder.const_i32(elem_idx)],
            "", /* void */
        );
    }

    pub(crate) fn table_grow(
        &self,
        ctxt: LLVMValueRef,
        table_idx: TableIdx,
        size: LLVMValueRef,
        value_to_fill: LLVMValueRef,
    ) -> LLVMValueRef {
        let func_type = Module::create_func_type(
            self.builder.i32(),
            &mut [
                self.builder.ptr(),
                self.builder.i32(),
                self.builder.i32(),
                self.builder.valtype2llvm(ValType::funcref()),
            ],
        );
        let table_grow_fn = self.get_rt_func("__wasmine_runtime.table_grow", func_type);
        self.builder.build_call(
            &table_grow_fn,
            &mut [ctxt, self.builder.const_i32(table_idx), size, value_to_fill],
            "table_grow_res",
        )
    }

    pub(crate) fn get_rt_ref(llvm_function: &Function) -> LLVMValueRef {
        llvm_function.get_param(0)
    }
}
