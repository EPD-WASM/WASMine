use super::numeric::{FRelationalOpConv, IRelationalOpConv};
use crate::{
    abstraction::{function::Function, module::Module},
    Context, TranslationError, Translator,
};
use llvm_sys::{
    prelude::{LLVMBasicBlockRef, LLVMTypeRef, LLVMValueRef},
    LLVMIntPredicate,
};
use module::{
    objects::{
        instruction::ControlInstruction,
        value::{Reference, Value, ValueRaw},
    },
    ModuleMetadata,
};
use std::{cell::RefCell, rc::Rc};
use wasm_types::{GlobalType, NumType, ValType};

pub(crate) struct LLVMInstructionConsumer<'wasm> {
    pub(crate) translator: Translator<'wasm>,
    llvm_functions: Rc<RefCell<Vec<Function>>>,
    error: Option<TranslationError>,
    terminator: Option<ControlInstruction>,
    wasm_module_meta: &'wasm module::ModuleMetadata,
    locals: Rc<RefCell<Vec<(LLVMValueRef, LLVMTypeRef)>>>,
    module: Rc<Module>,
    func_idx: usize,
    vars: Rc<RefCell<Vec<LLVMValueRef>>>,
}

impl<'wasm> LLVMInstructionConsumer<'wasm> {
    pub(crate) fn new(
        context: Rc<Context>,
        llvm_functions: Rc<RefCell<Vec<Function>>>,
        wasm_module_meta: &'wasm ModuleMetadata,
        module: Rc<Module>,
        locals: Rc<RefCell<Vec<(LLVMValueRef, LLVMTypeRef)>>>,
        vars: Rc<RefCell<Vec<LLVMValueRef>>>,
        func_idx: usize,
    ) -> Self {
        let translator = Translator {
            builder: context.create_builder(module.clone()),
            module: module.clone(),
            context: context.clone(),
            llvm_functions: llvm_functions.clone(),
            wasm_module_meta,
        };
        Self {
            translator,
            llvm_functions,
            error: None,
            terminator: None,
            wasm_module_meta,
            locals,
            module,
            vars,
            func_idx,
        }
    }

    pub(crate) fn set_basic_block(&self, bb: LLVMBasicBlockRef) {
        self.translator.builder.position_at_end(bb);
    }

    fn extend_vars(&mut self, idx: usize) {
        if self.vars.borrow().len() <= idx {
            self.vars
                .borrow_mut()
                .resize_with(idx + 1, || std::ptr::null_mut());
        }
    }
}

impl module::InstructionConsumer for LLVMInstructionConsumer<'_> {
    fn write_ibinary(&mut self, i: module::instructions::IBinaryInstruction) {
        self.extend_vars(i.out1);
        let mut vars = self.vars.borrow_mut();
        let lhs = vars[i.lhs];
        let rhs = vars[i.rhs];
        let out = &mut vars[i.out1];
        match self.translator.compile_ibinary(
            i,
            lhs,
            rhs,
            &self.llvm_functions.borrow()[self.func_idx],
        ) {
            Ok(v) => *out = v,
            Err(e) => self.error = Some(e),
        }
    }

    fn write_fbinary(&mut self, i: module::instructions::FBinaryInstruction) {
        self.extend_vars(i.out1);
        let mut vars = self.vars.borrow_mut();
        let lhs = vars[i.lhs];
        let rhs = vars[i.rhs];
        let out = &mut vars[i.out1];
        match self.translator.compile_fbinary(i, lhs, rhs) {
            Ok(v) => *out = v,
            Err(e) => self.error = Some(e),
        }
    }

    fn write_iunary(&mut self, i: module::instructions::IUnaryInstruction) {
        self.extend_vars(i.out1);
        let mut vars = self.vars.borrow_mut();
        let in1 = vars[i.in1];
        let out = &mut vars[i.out1];
        match self.translator.compile_iunary(i, in1) {
            Ok(v) => *out = v,
            Err(e) => self.error = Some(e),
        }
    }

    fn write_funary(&mut self, i: module::instructions::FUnaryInstruction) {
        self.extend_vars(i.out1);
        let mut vars = self.vars.borrow_mut();
        let in1 = vars[i.in1];
        let out = &mut vars[i.out1];
        match self.translator.compile_funary(i, in1) {
            Ok(v) => *out = v,
            Err(e) => self.error = Some(e),
        }
    }

    fn write_block(&mut self, i: module::instructions::Block) {
        self.finish(ControlInstruction::Block(i.block_type));
    }

    fn write_brif(&mut self, i: module::instructions::BrIf) {
        self.finish(ControlInstruction::BrIf(i.label_idx));
    }

    fn write_br(&mut self, i: module::instructions::Br) {
        self.finish(ControlInstruction::Br(i.label_idx));
    }

    fn write_br_table(&mut self, i: module::instructions::BrTable) {
        self.finish(ControlInstruction::BrTable(
            i.default_label_idx,
            i.label_indices,
        ));
    }

    fn write_call_indirect(&mut self, i: module::instructions::CallIndirect) {
        self.finish(ControlInstruction::CallIndirect(i.type_idx, i.table_idx));
    }

    fn write_call(&mut self, i: module::instructions::Call) {
        self.finish(ControlInstruction::Call(i.func_idx));
    }

    fn write_if_else(&mut self, i: module::instructions::IfElse) {
        self.finish(ControlInstruction::IfElse(i.block_type));
    }

    fn write_else(&mut self) {
        self.finish(ControlInstruction::Else);
    }

    fn write_loop(&mut self, i: module::instructions::Loop) {
        self.finish(ControlInstruction::Loop(i.block_type));
    }

    fn write_end(&mut self) {
        self.finish(ControlInstruction::End);
    }

    fn write_return(&mut self) {
        self.finish(ControlInstruction::Return);
    }

    fn write_unreachable(&mut self) {
        self.finish(ControlInstruction::Unreachable);
    }

    fn write_store(&mut self, i: module::instructions::StoreInstruction) {
        if let Err(e) = self.translator.compile_store(
            i,
            &mut self.vars.borrow_mut(),
            &self.llvm_functions.borrow()[self.func_idx],
        ) {
            self.error = Some(e);
        }
    }

    fn write_load(&mut self, i: module::instructions::LoadInstruction) {
        self.extend_vars(i.out1);
        if let Err(e) = self.translator.compile_load(
            i,
            &mut self.vars.borrow_mut(),
            &self.llvm_functions.borrow()[self.func_idx],
        ) {
            self.error = Some(e);
        }
    }

    fn write_test(&mut self, i: module::instructions::ITestInstruction) {
        self.extend_vars(i.out1);
        let mut vars = self.vars.borrow_mut();
        vars[i.out1] = self.translator.builder.build_icmp(
            LLVMIntPredicate::LLVMIntEQ,
            vars[i.in1],
            self.translator
                .builder
                .const_zero(ValType::Number(i.input_type)),
            "test_eqz",
        );
        vars[i.out1] = self.translator.builder.build_int_cast(
            vars[i.out1],
            self.translator.builder.i32(),
            false,
            "icmp -> i32",
        );
    }

    fn write_memory_size(&mut self, i: module::instructions::MemorySizeInstruction) {
        self.extend_vars(i.out1);
        self.vars.borrow_mut()[i.out1] = self.translator.ec_get_mem_size(
            Translator::get_rt_ref(&self.llvm_functions.borrow()[self.func_idx]),
            0,
        );
    }

    fn write_memory_grow(&mut self, i: module::instructions::MemoryGrowInstruction) {
        self.extend_vars(i.out1);
        let mut vars = self.vars.borrow_mut();
        vars[i.out1] = self.translator.memory_grow(
            Translator::get_rt_ref(&self.llvm_functions.borrow()[self.func_idx]),
            0,
            vars[i.in1],
        );
    }

    fn write_memory_copy(&mut self, i: module::instructions::MemoryCopyInstruction) {
        let vars = self.vars.borrow();
        self.translator.memory_copy(
            Translator::get_rt_ref(&self.llvm_functions.borrow()[self.func_idx]),
            0,
            vars[i.s],
            vars[i.d],
            vars[i.n],
        );
    }

    fn write_memory_fill(&mut self, i: module::instructions::MemoryFillInstruction) {
        let vars = self.vars.borrow();
        let value = self.translator.builder.build_int_cast(
            vars[i.val],
            self.translator.builder.i8(),
            false,
            "cast_fill_val",
        );
        self.translator.memory_fill(
            Translator::get_rt_ref(&self.llvm_functions.borrow()[self.func_idx]),
            0,
            vars[i.d],
            vars[i.n],
            value,
        );
    }

    fn write_memory_init(&mut self, i: module::instructions::MemoryInitInstruction) {
        let vars = self.vars.borrow();
        self.translator.memory_init(
            Translator::get_rt_ref(&self.llvm_functions.borrow()[self.func_idx]),
            0,
            i.data_idx,
            vars[i.s],
            vars[i.d],
            vars[i.n],
        );
    }

    fn write_data_drop(&mut self, i: module::instructions::DataDropInstruction) {
        self.translator.data_drop(
            Translator::get_rt_ref(&self.llvm_functions.borrow()[self.func_idx]),
            i.data_idx,
        );
    }

    fn write_trunc(&mut self, i: module::instructions::TruncInstruction) {
        self.extend_vars(i.out1);
        let mut vars = self.vars.borrow_mut();
        vars[i.out1] = self.translator.builder.build_fp2int_trunc(
            vars[i.in1],
            i.out1_type,
            i.in1_type,
            i.signed,
            "trunc",
            self.llvm_functions.borrow()[self.func_idx].get(),
        );
    }

    fn write_trunc_saturation(&mut self, i: module::instructions::TruncSaturationInstruction) {
        self.extend_vars(i.out1);
        let mut vars = self.vars.borrow_mut();
        let intrinsic_name = if i.signed {
            "llvm.fptosi.sat"
        } else {
            "llvm.fptoui.sat"
        };
        vars[i.out1] = self.translator.builder.call_unary_intrinsic(
            i.in1_type,
            i.out1_type,
            vars[i.in1],
            intrinsic_name,
            true,
        );
    }

    fn write_irelational(&mut self, i: module::instructions::IRelationalInstruction) {
        self.extend_vars(i.out1);
        let mut vars = self.vars.borrow_mut();
        vars[i.out1] = self.translator.builder.build_icmp(
            IRelationalOpConv(i.op).into(),
            vars[i.in1],
            vars[i.in2],
            "icmp",
        );
        vars[i.out1] = self.translator.builder.build_int_cast(
            vars[i.out1],
            self.translator.builder.i32(),
            false,
            "upcast icmp i8 -> i32",
        );
    }

    fn write_frelational(&mut self, i: module::instructions::FRelationalInstruction) {
        self.extend_vars(i.out1);
        let mut vars = self.vars.borrow_mut();
        vars[i.out1] = self.translator.builder.build_fcmp(
            FRelationalOpConv(i.op).into(),
            vars[i.in1],
            vars[i.in2],
            "fcmp",
        );
        vars[i.out1] = self.translator.builder.build_int_cast(
            vars[i.out1],
            self.translator.builder.i32(),
            false,
            "upcast fcmp i8 -> i32",
        );
    }

    fn write_wrap(&mut self, i: module::instructions::WrapInstruction) {
        self.extend_vars(i.out1);
        let mut vars = self.vars.borrow_mut();
        vars[i.out1] = self.translator.builder.build_int_cast(
            vars[i.in1],
            self.translator.builder.i32(),
            false,
            "wrap",
        );
    }

    fn write_convert(&mut self, i: module::instructions::ConvertInstruction) {
        self.extend_vars(i.out1);
        let mut vars = self.vars.borrow_mut();
        vars[i.out1] = self.translator.builder.build_int2float(
            vars[i.in1],
            self.translator
                .builder
                .valtype2llvm(ValType::Number(i.out1_type)),
            i.signed,
            "convert",
        );
    }

    fn write_reinterpret(&mut self, i: module::instructions::ReinterpretInstruction) {
        self.extend_vars(i.out1);
        let mut vars = self.vars.borrow_mut();
        vars[i.out1] = self.translator.builder.build_bitcast(
            vars[i.in1],
            self.translator
                .builder
                .valtype2llvm(ValType::Number(i.out1_type)),
            "reinterpret",
        );
    }

    fn write_extend_bits(&mut self, i: module::instructions::ExtendBitsInstruction) {
        self.extend_vars(i.out1);
        let mut vars = self.vars.borrow_mut();
        let actual_type = self.translator.builder.custom_type(i.input_size as u32);
        let part_view =
            self.translator
                .builder
                .build_int_cast(vars[i.in1], actual_type, false, "downcast");
        vars[i.out1] = self.translator.builder.build_int_cast(
            part_view,
            self.translator
                .builder
                .valtype2llvm(ValType::Number(i.out1_type)),
            true,
            "extendbits",
        );
    }

    fn write_extend_type(&mut self, i: module::instructions::ExtendTypeInstruction) {
        self.extend_vars(i.out1);
        let mut vars = self.vars.borrow_mut();
        vars[i.out1] = self.translator.builder.build_int_cast(
            vars[i.in1],
            self.translator.builder.i64(),
            i.signed,
            "extend",
        );
    }

    fn write_demote(&mut self, i: module::instructions::DemoteInstruction) {
        self.extend_vars(i.out1);
        let mut vars = self.vars.borrow_mut();
        vars[i.out1] = self.translator.builder.build_float_cast(
            vars[i.in1],
            self.translator.builder.f32(),
            "demote",
        );
    }

    fn write_promote(&mut self, i: module::instructions::PromoteInstruction) {
        self.extend_vars(i.out1);
        let mut vars = self.vars.borrow_mut();
        vars[i.out1] = self.translator.builder.call_funary_constrained_intrinsic(
            NumType::F32,
            NumType::F64,
            vars[i.in1],
            "llvm.experimental.constrained.fpext",
        )
    }

    fn write_const(&mut self, i: module::instructions::Constant) {
        self.extend_vars(i.out1);
        self.vars.borrow_mut()[i.out1] = match i.out1_type {
            NumType::I32 => self.translator.builder.const_i32(i.imm.into()),
            NumType::I64 => self.translator.builder.const_i64(i.imm.into()),
            NumType::F32 => self.translator.builder.const_f32(i.imm.into()),
            NumType::F64 => self.translator.builder.const_f64(i.imm.into()),
        };
    }

    fn write_reference_is_null(&mut self, i: module::instructions::ReferenceIsNullInstruction) {
        self.extend_vars(i.out1);
        let mut vars = self.vars.borrow_mut();
        vars[i.out1] = self.translator.builder.build_icmp(
            llvm_sys::LLVMIntPredicate::LLVMIntEQ,
            vars[i.in1],
            self.translator
                .builder
                .const_i64(ValueRaw::from(Value::Reference(Reference::Null)).as_u64()),
            "ref_is_null",
        );
        vars[i.out1] = self.translator.builder.build_int_cast(
            vars[i.out1],
            self.translator.builder.i32(),
            false,
            "icmp -> i32",
        );
    }

    fn write_reference_null(&mut self, i: module::instructions::ReferenceNullInstruction) {
        self.extend_vars(i.out1);
        self.vars.borrow_mut()[i.out1] = self
            .translator
            .builder
            .const_i64(ValueRaw::from(Value::Reference(Reference::Null)).as_u64());
    }

    fn write_reference_function(&mut self, i: module::instructions::ReferenceFunctionInstruction) {
        self.extend_vars(i.out1);
        self.vars.borrow_mut()[i.out1] = self.translator.builder.const_i64(i.func_idx as u64);
    }

    fn write_select(&mut self, i: module::instructions::SelectInstruction) {
        self.extend_vars(i.out1);
        let mut vars = self.vars.borrow_mut();
        let select_val = self.translator.builder.build_icmp(
            llvm_sys::LLVMIntPredicate::LLVMIntNE,
            vars[i.select_val],
            self.translator.builder.const_zero(ValType::i32()),
            "ToBool",
        );
        let select_res_val = self.translator.builder.build_select(
            select_val,
            vars[i.input_vals[0]],
            vars[i.input_vals[1]],
            "select",
        );
        vars[i.out1] = select_res_val;
    }

    fn write_table_set(&mut self, i: module::instructions::TableSetInstruction) {
        let vars = self.vars.borrow();
        self.translator.table_set(
            Translator::get_rt_ref(&self.llvm_functions.borrow()[self.func_idx]),
            i.table_idx,
            vars[i.in1],
            vars[i.idx],
        );
    }

    fn write_table_get(&mut self, i: module::instructions::TableGetInstruction) {
        self.extend_vars(i.out1);
        let mut vars = self.vars.borrow_mut();
        vars[i.out1] = self.translator.table_get(
            Translator::get_rt_ref(&self.llvm_functions.borrow()[self.func_idx]),
            i.table_idx,
            vars[i.idx],
            self.translator.builder.valtype2llvm(ValType::Reference(
                self.wasm_module_meta.tables[i.table_idx as usize]
                    .r#type
                    .ref_type,
            )),
        );
    }

    fn write_table_grow(&mut self, i: module::instructions::TableGrowInstruction) {
        self.extend_vars(i.out1);
        let mut vars = self.vars.borrow_mut();
        vars[i.out1] = self.translator.table_grow(
            Translator::get_rt_ref(&self.llvm_functions.borrow()[self.func_idx]),
            i.table_idx,
            vars[i.size],
            vars[i.value_to_fill],
        );
    }

    fn write_table_size(&mut self, i: module::instructions::TableSizeInstruction) {
        self.extend_vars(i.out1);
        self.vars.borrow_mut()[i.out1] = self.translator.table_size(
            Translator::get_rt_ref(&self.llvm_functions.borrow()[self.func_idx]),
            i.table_idx,
        );
    }

    fn write_table_fill(&mut self, i: module::instructions::TableFillInstruction) {
        let vars = self.vars.borrow();
        self.translator.table_fill(
            Translator::get_rt_ref(&self.llvm_functions.borrow()[self.func_idx]),
            i.table_idx,
            vars[i.i],
            vars[i.n],
            vars[i.ref_value],
        );
    }

    fn write_table_copy(&mut self, i: module::instructions::TableCopyInstruction) {
        let vars = self.vars.borrow();
        self.translator.table_copy(
            Translator::get_rt_ref(&self.llvm_functions.borrow()[self.func_idx]),
            i.table_idx_y,
            i.table_idx_x,
            vars[i.s],
            vars[i.d],
            vars[i.n],
        );
    }

    fn write_table_init(&mut self, i: module::instructions::TableInitInstruction) {
        let vars = self.vars.borrow();
        self.translator.table_init(
            Translator::get_rt_ref(&self.llvm_functions.borrow()[self.func_idx]),
            i.table_idx,
            i.elem_idx,
            vars[i.s],
            vars[i.d],
            vars[i.n],
        );
    }

    fn write_elem_drop(&mut self, i: module::instructions::ElemDropInstruction) {
        self.translator.elem_drop(
            Translator::get_rt_ref(&self.llvm_functions.borrow()[self.func_idx]),
            i.elem_idx,
        );
    }

    fn write_local_get(&mut self, i: module::instructions::LocalGetInstruction) {
        self.extend_vars(i.out1);
        let mut vars = self.vars.borrow_mut();
        vars[i.out1] = self.translator.builder.build_load(
            self.locals.borrow()[i.local_idx as usize].1,
            self.locals.borrow()[i.local_idx as usize].0,
            "local_get",
        );
    }

    fn write_global_get(&mut self, i: module::instructions::GlobalGetInstruction) {
        self.extend_vars(i.out1);
        let wasm_global = &self.wasm_module_meta.globals[i.global_idx as usize];
        let global_type = match wasm_global.r#type {
            GlobalType::Const(ty) => ty,
            GlobalType::Mut(ty) => ty,
        };
        let global_name = format!("__wasmine_global__{}", i.global_idx);
        let global = match self.module.get_global(&global_name) {
            Ok(g) => g,
            Err(e) => {
                self.error = Some(TranslationError::from(e));
                return;
            }
        };
        self.vars.borrow_mut()[i.out1] = self.translator.builder.build_load(
            self.translator.builder.valtype2llvm(global_type),
            global,
            "global_get",
        );
    }

    fn write_local_set(&mut self, i: module::instructions::LocalSetInstruction) {
        self.translator.builder.build_store(
            self.vars.borrow()[i.in1],
            self.locals.borrow()[i.local_idx as usize].0,
        );
    }

    fn write_global_set(&mut self, i: module::instructions::GlobalSetInstruction) {
        let global_name = format!("__wasmine_global__{}", i.global_idx);
        let global = match self.module.get_global(&global_name) {
            Ok(g) => g,
            Err(e) => {
                self.error = Some(TranslationError::from(e));
                return;
            }
        };
        self.translator
            .builder
            .build_store(self.vars.borrow()[i.in1], global);
    }

    fn write_local_tee(&mut self, i: module::instructions::LocalTeeInstruction) {
        self.extend_vars(i.out1);
        let mut vars = self.vars.borrow_mut();
        self.translator
            .builder
            .build_store(vars[i.in1], self.locals.borrow()[i.local_idx as usize].0);
        vars[i.out1] = vars[i.in1];
    }

    fn finish(&mut self, terminator: module::objects::instruction::ControlInstruction) {
        self.terminator = Some(terminator);
    }

    fn is_finished(&self) -> bool {
        self.terminator.is_some()
    }

    fn peek_terminator(&self) -> &module::objects::instruction::ControlInstruction {
        self.terminator.as_ref().unwrap()
    }
}
