use super::{context::Context, function::Function, module::Module};
use crate::util::c_str;
use ir::structs::value::ValueRaw;
use llvm_sys::{
    core::{
        LLVMAddIncoming, LLVMAppendBasicBlockInContext, LLVMArrayType2, LLVMBuildAdd,
        LLVMBuildAggregateRet, LLVMBuildAlloca, LLVMBuildBitCast, LLVMBuildBr, LLVMBuildCall2,
        LLVMBuildCondBr, LLVMBuildFCmp, LLVMBuildFPCast, LLVMBuildFPToSI, LLVMBuildFPToUI,
        LLVMBuildGEP2, LLVMBuildICmp, LLVMBuildIntCast2, LLVMBuildLoad2, LLVMBuildMul,
        LLVMBuildPhi, LLVMBuildRet, LLVMBuildRetVoid, LLVMBuildSIToFP, LLVMBuildSelect,
        LLVMBuildStore, LLVMBuildSub, LLVMBuildSwitch, LLVMBuildUIToFP, LLVMBuildUnreachable,
        LLVMConstBitCast, LLVMConstInt, LLVMConstNull, LLVMConstReal, LLVMCreateBuilderInContext,
        LLVMDisposeBuilder, LLVMDoubleTypeInContext, LLVMFloatTypeInContext,
        LLVMInt16TypeInContext, LLVMInt1Type, LLVMInt32TypeInContext, LLVMInt64TypeInContext,
        LLVMInt8TypeInContext, LLVMIntTypeInContext, LLVMMDStringInContext2, LLVMMetadataAsValue,
        LLVMMetadataTypeInContext, LLVMPointerType, LLVMPointerTypeInContext,
        LLVMPositionBuilderAtEnd, LLVMStructTypeInContext, LLVMTypeOf, LLVMVoidTypeInContext,
    },
    prelude::{LLVMBasicBlockRef, LLVMBuilderRef, LLVMContextRef, LLVMTypeRef, LLVMValueRef},
    LLVMIntPredicate, LLVMRealPredicate,
};
use std::{rc::Rc, sync::OnceLock};
use wasm_types::{NumType, ValType};

struct LLVMTypeWrapper(LLVMTypeRef);
unsafe impl Send for LLVMTypeWrapper {}
unsafe impl Sync for LLVMTypeWrapper {}

pub(crate) struct LLVMValueRefWrapper(pub(crate) LLVMValueRef);
unsafe impl Send for LLVMValueRefWrapper {}
unsafe impl Sync for LLVMValueRefWrapper {}

#[allow(non_snake_case)]
pub(crate) struct Builder {
    inner: LLVMBuilderRef,
    context: LLVMContextRef,
    module: Rc<Module>,

    I1: OnceLock<LLVMTypeWrapper>,
    I8: OnceLock<LLVMTypeWrapper>,
    I16: OnceLock<LLVMTypeWrapper>,
    I32: OnceLock<LLVMTypeWrapper>,
    I64: OnceLock<LLVMTypeWrapper>,
    F32: OnceLock<LLVMTypeWrapper>,
    F64: OnceLock<LLVMTypeWrapper>,
    PTR: OnceLock<LLVMTypeWrapper>,
    VOID: OnceLock<LLVMTypeWrapper>,
    METADATA: OnceLock<LLVMTypeWrapper>,
    VEC: OnceLock<LLVMTypeWrapper>,

    pub(crate) DEFAULT_ROUNDING_MODE: OnceLock<LLVMValueRefWrapper>,
    pub(crate) DEFAULT_FP_EXCEPTION_MODE: OnceLock<LLVMValueRefWrapper>,
}

impl Builder {
    pub(crate) fn create(context: &Context, module: Rc<Module>) -> Self {
        Self {
            inner: unsafe { LLVMCreateBuilderInContext(context.get()) },
            context: context.get(),
            module,

            I1: OnceLock::new(),
            I8: OnceLock::new(),
            I16: OnceLock::new(),
            I32: OnceLock::new(),
            I64: OnceLock::new(),
            F32: OnceLock::new(),
            F64: OnceLock::new(),
            PTR: OnceLock::new(),
            VOID: OnceLock::new(),
            METADATA: OnceLock::new(),
            VEC: OnceLock::new(),

            DEFAULT_ROUNDING_MODE: OnceLock::new(),
            DEFAULT_FP_EXCEPTION_MODE: OnceLock::new(),
        }
    }

    pub(crate) fn get(&self) -> LLVMBuilderRef {
        self.inner
    }

    pub(crate) fn i1(&self) -> LLVMTypeRef {
        self.I1
            .get_or_init(|| unsafe { LLVMTypeWrapper(LLVMInt1Type()) })
            .0
    }

    pub(crate) fn i8(&self) -> LLVMTypeRef {
        self.I8
            .get_or_init(|| unsafe { LLVMTypeWrapper(LLVMInt8TypeInContext(self.context)) })
            .0
    }

    pub(crate) fn i16(&self) -> LLVMTypeRef {
        self.I16
            .get_or_init(|| unsafe { LLVMTypeWrapper(LLVMInt16TypeInContext(self.context)) })
            .0
    }

    pub(crate) fn i32(&self) -> LLVMTypeRef {
        self.I32
            .get_or_init(|| unsafe { LLVMTypeWrapper(LLVMInt32TypeInContext(self.context)) })
            .0
    }

    pub(crate) fn i64(&self) -> LLVMTypeRef {
        self.I64
            .get_or_init(|| unsafe { LLVMTypeWrapper(LLVMInt64TypeInContext(self.context)) })
            .0
    }

    pub(crate) fn f32(&self) -> LLVMTypeRef {
        self.F32
            .get_or_init(|| unsafe { LLVMTypeWrapper(LLVMFloatTypeInContext(self.context)) })
            .0
    }

    pub(crate) fn f64(&self) -> LLVMTypeRef {
        self.F64
            .get_or_init(|| unsafe { LLVMTypeWrapper(LLVMDoubleTypeInContext(self.context)) })
            .0
    }

    pub(crate) fn ptr(&self) -> LLVMTypeRef {
        self.PTR
            .get_or_init(|| unsafe { LLVMTypeWrapper(LLVMPointerTypeInContext(self.context, 0)) })
            .0
    }

    pub(crate) fn void(&self) -> LLVMTypeRef {
        self.VOID
            .get_or_init(|| unsafe { LLVMTypeWrapper(LLVMVoidTypeInContext(self.context)) })
            .0
    }

    pub(crate) fn metadata(&self) -> LLVMTypeRef {
        self.METADATA
            .get_or_init(|| unsafe { LLVMTypeWrapper(LLVMMetadataTypeInContext(self.context)) })
            .0
    }

    pub(crate) fn vec(&self) -> LLVMTypeRef {
        self.VEC
            .get_or_init(|| unsafe { LLVMTypeWrapper(LLVMPointerTypeInContext(self.context, 0)) })
            .0
    }

    pub(crate) fn value_raw_ty(&self) -> LLVMTypeRef {
        unsafe { LLVMIntTypeInContext(self.context, (std::mem::size_of::<ValueRaw>() * 8) as u32) }
    }

    pub(crate) fn r#struct(&self, elems: &mut [LLVMTypeRef]) -> LLVMTypeRef {
        unsafe {
            LLVMStructTypeInContext(
                self.context,
                elems.as_mut_ptr(),
                elems.len() as u32,
                false.into(),
            )
        }
    }

    pub(crate) fn array(&self, elem_ty: LLVMTypeRef, len: usize) -> LLVMTypeRef {
        unsafe { LLVMArrayType2(elem_ty, len as u64) }
    }

    pub(crate) fn valtype2llvm(&self, valtype: ValType) -> LLVMTypeRef {
        match valtype {
            ValType::Number(NumType::F32) => self.f32(),
            ValType::Number(NumType::F64) => self.f64(),
            ValType::Number(NumType::I32) => self.i32(),
            ValType::Number(NumType::I64) => self.i64(),
            ValType::Reference(_) => self.i64(),
            ValType::VecType => self.vec(),
        }
    }

    pub(crate) fn const_zero(&self, valtype: ValType) -> LLVMValueRef {
        unsafe {
            match valtype {
                ValType::Number(NumType::F32) => LLVMConstNull(self.f32()),
                ValType::Number(NumType::F64) => LLVMConstNull(self.f64()),
                ValType::Number(NumType::I32) => LLVMConstNull(self.i32()),
                ValType::Number(NumType::I64) => LLVMConstNull(self.i64()),
                ValType::Reference(_) => LLVMConstNull(self.ptr()),
                ValType::VecType => LLVMConstNull(self.vec()),
            }
        }
    }

    pub(crate) fn const_i32(&self, i: u32) -> LLVMValueRef {
        unsafe { LLVMConstInt(self.i32(), i as u64, false.into()) }
    }
    pub(crate) fn const_i64(&self, i: u64) -> LLVMValueRef {
        unsafe { LLVMConstInt(self.i64(), i, false.into()) }
    }
    pub(crate) fn const_f32(&self, f: f32) -> LLVMValueRef {
        // current rust version does invalid conversion f32 -> f64 for signalling NaNs
        // => just to be sure, we use transfer the bits directly without up- / downcasting
        unsafe {
            let bits = LLVMConstInt(self.i32(), f.to_bits() as u64, false.into());
            LLVMConstBitCast(bits, self.f32())
        }
    }
    pub(crate) fn const_f64(&self, f: f64) -> LLVMValueRef {
        unsafe { LLVMConstReal(self.f64(), f) }
    }
    pub(crate) fn const_i1(&self, i: bool) -> LLVMValueRef {
        unsafe { LLVMConstInt(self.i1(), i as u64, false.into()) }
    }

    pub(crate) fn build_gep(
        &self,
        ty: LLVMTypeRef,
        base: LLVMValueRef,
        indices: &mut [LLVMValueRef],
        name: &str,
    ) -> LLVMValueRef {
        unsafe {
            LLVMBuildGEP2(
                self.get(),
                ty,
                base,
                indices.as_mut_ptr(),
                indices.len() as u32,
                c_str(name).as_ptr(),
            )
        }
    }

    pub(crate) fn build_load(
        &self,
        ty: LLVMTypeRef,
        ptr: LLVMValueRef,
        name: &str,
    ) -> LLVMValueRef {
        unsafe { LLVMBuildLoad2(self.get(), ty, ptr, c_str(name).as_ptr()) }
    }

    pub(crate) fn build_alloca(&self, ty: LLVMTypeRef, name: &str) -> LLVMValueRef {
        let res = unsafe { LLVMBuildAlloca(self.get(), ty, c_str(name).as_ptr()) };
        // somehow, the result is not a pointer (TODO: ask alexis why this could be / how to debug)
        self.build_bitcast(res, unsafe { LLVMPointerType(ty, 0) }, "local_ptr")
    }

    pub(crate) fn build_int_cast(
        &self,
        val: LLVMValueRef,
        to_ty: LLVMTypeRef,
        signed: bool,
        name: &str,
    ) -> LLVMValueRef {
        unsafe { LLVMBuildIntCast2(self.get(), val, to_ty, signed.into(), c_str(name).as_ptr()) }
    }

    pub(crate) fn build_int2float(
        &self,
        val: LLVMValueRef,
        to_ty: LLVMTypeRef,
        signed: bool,
        name: &str,
    ) -> LLVMValueRef {
        if signed {
            unsafe { LLVMBuildSIToFP(self.get(), val, to_ty, c_str(name).as_ptr()) }
        } else {
            unsafe { LLVMBuildUIToFP(self.get(), val, to_ty, c_str(name).as_ptr()) }
        }
    }

    pub(crate) fn build_float_cast(
        &self,
        val: LLVMValueRef,
        to_ty: LLVMTypeRef,
        name: &str,
    ) -> LLVMValueRef {
        unsafe { LLVMBuildFPCast(self.get(), val, to_ty, c_str(name).as_ptr()) }
    }

    pub(crate) fn build_bitcast(
        &self,
        val: LLVMValueRef,
        to_ty: LLVMTypeRef,
        name: &str,
    ) -> LLVMValueRef {
        unsafe { LLVMBuildBitCast(self.get(), val, to_ty, c_str(name).as_ptr()) }
    }

    pub(crate) fn build_store(&self, val: LLVMValueRef, ptr: LLVMValueRef) {
        unsafe {
            LLVMBuildStore(self.get(), val, ptr);
        }
    }

    pub(crate) fn build_ret(&self, val: LLVMValueRef) {
        unsafe {
            LLVMBuildRet(self.get(), val);
        }
    }

    pub(crate) fn build_ret_void(&self) {
        unsafe {
            LLVMBuildRetVoid(self.get());
        }
    }

    pub(crate) fn build_aggregate_ret(&self, vals: &mut [LLVMValueRef]) {
        unsafe {
            LLVMBuildAggregateRet(self.get(), vals.as_mut_ptr(), vals.len() as u32);
        }
    }

    pub(crate) fn build_unconditional_branch(&self, block: LLVMBasicBlockRef) {
        unsafe {
            LLVMBuildBr(self.get(), block);
        }
    }

    pub(crate) fn position_at_end(&self, block: LLVMBasicBlockRef) {
        unsafe {
            LLVMPositionBuilderAtEnd(self.get(), block);
        }
    }

    pub(crate) fn build_icmp(
        &self,
        op: LLVMIntPredicate,
        lhs: LLVMValueRef,
        rhs: LLVMValueRef,
        name: &str,
    ) -> LLVMValueRef {
        unsafe { LLVMBuildICmp(self.get(), op, lhs, rhs, c_str(name).as_ptr()) }
    }

    pub(crate) fn build_fcmp(
        &self,
        op: LLVMRealPredicate,
        lhs: LLVMValueRef,
        rhs: LLVMValueRef,
        name: &str,
    ) -> LLVMValueRef {
        unsafe { LLVMBuildFCmp(self.get(), op, lhs, rhs, c_str(name).as_ptr()) }
    }

    pub(crate) fn build_conditional_branch(
        &self,
        cond: LLVMValueRef,
        then_block: LLVMBasicBlockRef,
        else_block: LLVMBasicBlockRef,
    ) {
        unsafe {
            LLVMBuildCondBr(self.get(), cond, then_block, else_block);
        }
    }

    pub(crate) fn build_call(
        &self,
        func: &Function,
        args: &mut [LLVMValueRef],
        name: &str,
    ) -> LLVMValueRef {
        unsafe {
            LLVMBuildCall2(
                self.get(),
                func.r#type(),
                func.get(),
                args.as_mut_ptr(),
                args.len() as u32,
                c_str(name).as_ptr(),
            )
        }
    }

    pub(crate) fn build_unreachable(&self) {
        let intrinsic_function = self
            .module
            .get_intrinsic_func("llvm.trap", &mut [], self.void())
            .unwrap();
        self.build_call(&intrinsic_function, &mut [], "");
        unsafe {
            LLVMBuildUnreachable(self.get());
        }
    }

    pub(crate) fn build_switch(
        &self,
        selector: LLVMValueRef,
        default_block: LLVMBasicBlockRef,
        num_cases: u32,
    ) -> LLVMValueRef {
        unsafe { LLVMBuildSwitch(self.get(), selector, default_block, num_cases) }
    }

    pub(crate) fn build_select(
        &self,
        cond: LLVMValueRef,
        then_val: LLVMValueRef,
        else_val: LLVMValueRef,
        name: &str,
    ) -> LLVMValueRef {
        unsafe { LLVMBuildSelect(self.get(), cond, then_val, else_val, c_str(name).as_ptr()) }
    }

    pub(crate) fn build_phi(&self, ty: LLVMTypeRef, name: &str) -> LLVMValueRef {
        unsafe { LLVMBuildPhi(self.get(), ty, c_str(name).as_ptr()) }
    }

    pub(crate) fn phi_add_incoming(
        phi: LLVMValueRef,
        vals: &mut [LLVMValueRef],
        blocks: &mut [LLVMBasicBlockRef],
    ) {
        unsafe {
            LLVMAddIncoming(
                phi,
                vals.as_mut_ptr(),
                blocks.as_mut_ptr(),
                blocks.len() as u32,
            );
        }
    }

    pub(crate) fn build_fp2int_trunc(
        &self,
        val: LLVMValueRef,
        out_ty: NumType,
        in_ty: NumType,
        signed: bool,
        name: &str,
        llvm_func: LLVMValueRef,
    ) -> LLVMValueRef {
        // TODO: define this globally per function
        let trap_block = unsafe {
            LLVMAppendBasicBlockInContext(self.context, llvm_func, c_str("trap").as_ptr())
        };
        let cont_block = unsafe {
            LLVMAppendBasicBlockInContext(self.context, llvm_func, c_str("cont").as_ptr())
        };
        let is_nan_or_inf = self.build_fcmp(
            LLVMRealPredicate::LLVMRealUNO,
            val,
            val,
            "check_is_nan_or_inf",
        );

        // case: input is not NaN
        let max_allowed_val = match (in_ty, out_ty, signed) {
            (NumType::F32, NumType::I32, true) => self.const_f32(i32::MAX as f32),
            (NumType::F32, NumType::I32, false) => self.const_f32(u32::MAX as f32),
            (NumType::F32, NumType::I64, true) => self.const_f32(i64::MAX as f32),
            (NumType::F32, NumType::I64, false) => self.const_f32(u64::MAX as f32),
            (NumType::F64, NumType::I32, true) => self.const_f64(i32::MAX as f64),
            (NumType::F64, NumType::I32, false) => self.const_f64(u32::MAX as f64),
            (NumType::F64, NumType::I64, true) => self.const_f64(i64::MAX as f64),
            (NumType::F64, NumType::I64, false) => self.const_f64(u64::MAX as f64),
            _ => unreachable!(),
        };
        let is_too_large = self.build_fcmp(
            LLVMRealPredicate::LLVMRealOGE,
            val,
            max_allowed_val,
            "check_is_too_large",
        );

        // case: input is not too large & not NaN

        /*
           You may now ask yourself: why these weird numbers?
           We need to check whether the supplied float, when converted to an integer, is out of the range
           of our target integer type. With test here, whether the float is too small, meaning less than the
           most negative number representable by the target integer type. However, float truncation (which is applied by
           llvm under the hood) automatically rounds towards zero. This means we can be anything higher than the smallest
           integer number minus one, e.g. -0.9 for f32 -> u32.

           But why the -130.0? This is a magic number which is close enough to the f32 epsilon at this point to get the next
           smaller, representable number. A cleaner way would be to use something like c++ std::nexttoward, but rust
           doesn't support this yet.
        */
        let min_allowed_val = match (in_ty, out_ty, signed) {
            (NumType::F32, NumType::I32, true) => self.const_f32(i32::MIN as f32 - 130.0),
            (NumType::F32, NumType::I32, false) => self.const_f32(u32::MIN as f32 - 1.0),
            (NumType::F32, NumType::I64, true) => self.const_f32(i64::MIN as f32),
            (NumType::F32, NumType::I64, false) => self.const_f32(u64::MIN as f32 - 1.0),
            (NumType::F64, NumType::I32, true) => self.const_f64(i32::MIN as f64),
            (NumType::F64, NumType::I32, false) => self.const_f64(u32::MIN as f64 - 1.0),
            (NumType::F64, NumType::I64, true) => self.const_f64(i64::MIN as f64),
            (NumType::F64, NumType::I64, false) => self.const_f64(u64::MIN as f64 - 1.0),
            _ => unreachable!(),
        };
        let is_too_small = self.build_fcmp(
            LLVMRealPredicate::LLVMRealOLE,
            val,
            min_allowed_val,
            "check_is_too_small",
        );

        let cond = self.build_or(is_nan_or_inf, is_too_large, "or");
        let cond = self.build_or(cond, is_too_small, "or");
        self.build_conditional_branch(cond, trap_block, cont_block);

        // case: invalid conversion
        self.position_at_end(trap_block);
        self.build_unreachable();

        // case: input is not too small & not too large & not NaN
        self.position_at_end(cont_block);
        let out_ty = self.valtype2llvm(ValType::Number(out_ty));
        if signed {
            unsafe { LLVMBuildFPToSI(self.get(), val, out_ty, c_str(name).as_ptr()) }
        } else {
            unsafe { LLVMBuildFPToUI(self.get(), val, out_ty, c_str(name).as_ptr()) }
        }
    }

    pub(crate) unsafe fn call_unary_intrinsic(
        &self,
        param_ty: NumType,
        ret_ty: NumType,
        param: LLVMValueRef,
        intrinsic_name: &str,
        mangling_needs_return_ty: bool,
    ) -> LLVMValueRef {
        let mangled_name = if mangling_needs_return_ty {
            format!("{intrinsic_name}.{ret_ty}.{param_ty}",)
        } else {
            format!("{intrinsic_name}.{param_ty}",)
        };
        let intrinsic_function = self
            .module
            .get_intrinsic_func(
                &mangled_name,
                &mut [self.valtype2llvm(ValType::Number(param_ty))],
                self.valtype2llvm(ValType::Number(ret_ty)),
            )
            .unwrap();
        self.build_call(&intrinsic_function, &mut [param], "call_intrinsic")
    }

    pub(crate) unsafe fn call_binary_intrinsic(
        &self,
        param_tys: [NumType; 2],
        params: [LLVMValueRef; 2],
        ret_ty: NumType,
        intrinsic_name: &str,
    ) -> LLVMValueRef {
        let mangled_name = format!("{}.{}", intrinsic_name, param_tys[0]);
        self.call_binary_intrinsic_raw(
            [
                self.valtype2llvm(ValType::Number(param_tys[0])),
                self.valtype2llvm(ValType::Number(param_tys[1])),
            ],
            params,
            self.valtype2llvm(ValType::Number(ret_ty)),
            &mangled_name,
        )
    }

    pub(crate) unsafe fn call_fbinary_constrained_intrinsic(
        &self,
        ty: NumType,
        params: [LLVMValueRef; 2],
        intrinsic_name: &str,
    ) -> LLVMValueRef {
        let mangled_name = format!("{intrinsic_name}.{ty}",);
        let ty = self.valtype2llvm(ValType::Number(ty));
        let intrinsic_function = self
            .module
            .get_intrinsic_func(
                &mangled_name,
                &mut [ty, ty, self.metadata(), self.metadata()],
                ty,
            )
            .unwrap();

        let rounding_mode = self.DEFAULT_ROUNDING_MODE.get_or_init(|| {
            let rm = "round.dynamic";
            let rm = LLVMMDStringInContext2(self.context, c_str(rm).as_ptr(), rm.len());
            LLVMValueRefWrapper(LLVMMetadataAsValue(self.context, rm))
        });

        let fp_exception_mode = self.DEFAULT_FP_EXCEPTION_MODE.get_or_init(|| {
            let fp_e_m = "fpexcept.ignore";
            let fp_e_m = LLVMMDStringInContext2(self.context, c_str(fp_e_m).as_ptr(), fp_e_m.len());
            LLVMValueRefWrapper(LLVMMetadataAsValue(self.context, fp_e_m))
        });

        self.build_call(
            &intrinsic_function,
            &mut [params[0], params[1], rounding_mode.0, fp_exception_mode.0],
            "call_intrinsic",
        )
    }

    pub(crate) unsafe fn call_funary_constrained_intrinsic(
        &self,
        in_ty: NumType,
        out_ty: NumType,
        param: LLVMValueRef,
        intrinsic_name: &str,
    ) -> LLVMValueRef {
        let mangled_name = format!("{intrinsic_name}.{out_ty}.{in_ty}");
        let in_ty = self.valtype2llvm(ValType::Number(in_ty));
        let out_ty = self.valtype2llvm(ValType::Number(out_ty));
        let intrinsic_function = self
            .module
            .get_intrinsic_func(&mangled_name, &mut [in_ty, self.metadata()], out_ty)
            .unwrap();

        let fp_exception_mode = self.DEFAULT_FP_EXCEPTION_MODE.get_or_init(|| {
            let fp_e_m = "fpexcept.ignore";
            let fp_e_m = LLVMMDStringInContext2(self.context, c_str(fp_e_m).as_ptr(), fp_e_m.len());
            LLVMValueRefWrapper(LLVMMetadataAsValue(self.context, fp_e_m))
        });

        self.build_call(
            &intrinsic_function,
            &mut [param, fp_exception_mode.0],
            "call_intrinsic",
        )
    }

    pub(crate) unsafe fn call_binary_intrinsic_raw(
        &self,
        mut param_tys: [LLVMTypeRef; 2],
        mut params: [LLVMValueRef; 2],
        ret_ty: LLVMTypeRef,
        intrinsic_name: &str,
    ) -> LLVMValueRef {
        let intrinsic_function = self
            .module
            .get_intrinsic_func(intrinsic_name, &mut param_tys, ret_ty)
            .unwrap();
        self.build_call(&intrinsic_function, &mut params, "call_intrinsic")
    }

    pub(crate) fn build_add(
        &self,
        lhs: LLVMValueRef,
        rhs: LLVMValueRef,
        name: &str,
    ) -> LLVMValueRef {
        unsafe { LLVMBuildAdd(self.get(), lhs, rhs, c_str(name).as_ptr()) }
    }

    pub(crate) fn build_sub(
        &self,
        lhs: LLVMValueRef,
        rhs: LLVMValueRef,
        name: &str,
    ) -> LLVMValueRef {
        unsafe { LLVMBuildSub(self.get(), lhs, rhs, c_str(name).as_ptr()) }
    }

    pub(crate) fn build_mul(
        &self,
        lhs: LLVMValueRef,
        rhs: LLVMValueRef,
        name: &str,
    ) -> LLVMValueRef {
        unsafe { LLVMBuildMul(self.get(), lhs, rhs, c_str(name).as_ptr()) }
    }

    pub(crate) fn build_trap_if_is_zero(&self, val: LLVMValueRef, llvm_func: LLVMValueRef) {
        let icmp_res = self.build_icmp(
            LLVMIntPredicate::LLVMIntEQ,
            val,
            unsafe { LLVMConstNull(LLVMTypeOf(val)) },
            "check_is_zero",
        );
        // TODO: define this globally per function == trap block
        let is_zero = unsafe {
            LLVMAppendBasicBlockInContext(self.context, llvm_func, c_str("is_zero").as_ptr())
        };
        let is_not_zero = unsafe {
            LLVMAppendBasicBlockInContext(self.context, llvm_func, c_str("is_not_zero").as_ptr())
        };
        self.build_conditional_branch(icmp_res, is_zero, is_not_zero);

        // case: val == 0
        self.position_at_end(is_zero);
        self.build_unreachable();

        // case: val != 0
        self.position_at_end(is_not_zero);
    }

    pub(crate) fn build_udiv(
        &self,
        lhs: LLVMValueRef,
        rhs: LLVMValueRef,
        name: &str,
        llvm_func: LLVMValueRef,
    ) -> LLVMValueRef {
        self.build_trap_if_is_zero(rhs, llvm_func);
        unsafe { llvm_sys::core::LLVMBuildUDiv(self.get(), lhs, rhs, c_str(name).as_ptr()) }
    }

    pub(crate) fn build_sdiv(
        &self,
        lhs: LLVMValueRef,
        rhs: LLVMValueRef,
        name: &str,
        llvm_func: LLVMValueRef,
    ) -> LLVMValueRef {
        self.build_trap_if_is_zero(rhs, llvm_func);
        unsafe { llvm_sys::core::LLVMBuildSDiv(self.get(), lhs, rhs, c_str(name).as_ptr()) }
    }

    pub(crate) fn build_urem(
        &self,
        lhs: LLVMValueRef,
        rhs: LLVMValueRef,
        name: &str,
        llvm_func: LLVMValueRef,
    ) -> LLVMValueRef {
        self.build_trap_if_is_zero(rhs, llvm_func);
        unsafe { llvm_sys::core::LLVMBuildURem(self.get(), lhs, rhs, c_str(name).as_ptr()) }
    }

    pub(crate) fn build_srem(
        &self,
        lhs: LLVMValueRef,
        rhs: LLVMValueRef,
        name: &str,
        llvm_func: LLVMValueRef,
    ) -> LLVMValueRef {
        self.build_trap_if_is_zero(rhs, llvm_func);
        unsafe { llvm_sys::core::LLVMBuildSRem(self.get(), lhs, rhs, c_str(name).as_ptr()) }
    }

    pub(crate) fn build_and(
        &self,
        lhs: LLVMValueRef,
        rhs: LLVMValueRef,
        name: &str,
    ) -> LLVMValueRef {
        unsafe { llvm_sys::core::LLVMBuildAnd(self.get(), lhs, rhs, c_str(name).as_ptr()) }
    }

    pub(crate) fn build_or(
        &self,
        lhs: LLVMValueRef,
        rhs: LLVMValueRef,
        name: &str,
    ) -> LLVMValueRef {
        unsafe { llvm_sys::core::LLVMBuildOr(self.get(), lhs, rhs, c_str(name).as_ptr()) }
    }

    pub(crate) fn build_xor(
        &self,
        lhs: LLVMValueRef,
        rhs: LLVMValueRef,
        name: &str,
    ) -> LLVMValueRef {
        unsafe { llvm_sys::core::LLVMBuildXor(self.get(), lhs, rhs, c_str(name).as_ptr()) }
    }

    pub(crate) fn build_shl(
        &self,
        lhs: LLVMValueRef,
        rhs: LLVMValueRef,
        name: &str,
    ) -> LLVMValueRef {
        unsafe { llvm_sys::core::LLVMBuildShl(self.get(), lhs, rhs, c_str(name).as_ptr()) }
    }

    pub(crate) fn build_lshr(
        &self,
        lhs: LLVMValueRef,
        rhs: LLVMValueRef,
        name: &str,
    ) -> LLVMValueRef {
        unsafe { llvm_sys::core::LLVMBuildLShr(self.get(), lhs, rhs, c_str(name).as_ptr()) }
    }

    pub(crate) fn build_ashr(
        &self,
        lhs: LLVMValueRef,
        rhs: LLVMValueRef,
        name: &str,
    ) -> LLVMValueRef {
        unsafe { llvm_sys::core::LLVMBuildAShr(self.get(), lhs, rhs, c_str(name).as_ptr()) }
    }

    // TODO: use intrinsic for this
    pub(crate) fn build_rotl(
        &self,
        lhs: LLVMValueRef,
        rhs: LLVMValueRef,
        types: NumType,
    ) -> LLVMValueRef {
        unsafe {
            let val = lhs;
            let rotate_by = rhs;

            let left_shift =
                llvm_sys::core::LLVMBuildShl(self.get(), val, rotate_by, c_str("rotl").as_ptr());
            let leftover_bits_cnt = match types {
                NumType::I32 => {
                    let bits = llvm_sys::core::LLVMConstInt(self.i32(), 32, 0);
                    llvm_sys::core::LLVMBuildSub(
                        self.get(),
                        bits,
                        rotate_by,
                        c_str("rotl").as_ptr(),
                    )
                }
                NumType::I64 => {
                    let bits = llvm_sys::core::LLVMConstInt(self.i64(), 64, 0);
                    llvm_sys::core::LLVMBuildSub(
                        self.get(),
                        bits,
                        rotate_by,
                        c_str("rotl").as_ptr(),
                    )
                }
                _ => panic!("parser error, expected i32 or i64"),
            };
            let right_shift = llvm_sys::core::LLVMBuildLShr(
                self.get(),
                val,
                leftover_bits_cnt,
                c_str("rotl").as_ptr(),
            );
            llvm_sys::core::LLVMBuildOr(self.get(), left_shift, right_shift, c_str("rotl").as_ptr())
        }
    }

    // TODO: use intrinsic for this
    pub(crate) fn build_rotr(
        &self,
        lhs: LLVMValueRef,
        rhs: LLVMValueRef,
        types: NumType,
    ) -> LLVMValueRef {
        unsafe {
            let val = lhs;
            let rotate_by = rhs;

            let right_shift =
                llvm_sys::core::LLVMBuildLShr(self.get(), val, rotate_by, c_str("rotr").as_ptr());
            let leftover_bits_cnt = match types {
                NumType::I32 => {
                    let bits = llvm_sys::core::LLVMConstInt(self.i32(), 32, 0);
                    llvm_sys::core::LLVMBuildSub(
                        self.get(),
                        bits,
                        rotate_by,
                        c_str("rotr").as_ptr(),
                    )
                }
                NumType::I64 => {
                    let bits = llvm_sys::core::LLVMConstInt(self.i64(), 64, 0);
                    llvm_sys::core::LLVMBuildSub(
                        self.get(),
                        bits,
                        rotate_by,
                        c_str("rotr").as_ptr(),
                    )
                }
                _ => panic!("parser error, expected i32 or i64"),
            };
            let left_shift = llvm_sys::core::LLVMBuildShl(
                self.get(),
                val,
                leftover_bits_cnt,
                c_str("rotr").as_ptr(),
            );
            llvm_sys::core::LLVMBuildOr(self.get(), left_shift, right_shift, c_str("rotr").as_ptr())
        }
    }
}

impl Drop for Builder {
    fn drop(&mut self) {
        unsafe {
            LLVMDisposeBuilder(self.inner);
        }
    }
}
