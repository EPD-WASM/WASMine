use crate::abstraction::function::Function;
use crate::{error::TranslationError, translator::Translator};
use llvm_sys::prelude::LLVMValueRef;
use llvm_sys::{LLVMIntPredicate, LLVMRealPredicate};
use module::instructions::*;
use module::{
    instructions::{FBinaryInstruction, FUnaryInstruction, IBinaryInstruction, IUnaryInstruction},
    InstructionDecoder,
};
use wasm_types::{
    ConversionOp, FBinaryOp, FRelationalOp, FUnaryOp, IBinaryOp, IRelationalOp, ITestOp, IUnaryOp,
    InstructionType, NumType, NumericInstructionCategory, ValType,
};

pub(super) struct IRelationalOpConv(pub(super) IRelationalOp);
impl From<IRelationalOpConv> for LLVMIntPredicate {
    fn from(val: IRelationalOpConv) -> Self {
        match val.0 {
            IRelationalOp::Eq => LLVMIntPredicate::LLVMIntEQ,
            IRelationalOp::Ne => LLVMIntPredicate::LLVMIntNE,
            IRelationalOp::LtS => LLVMIntPredicate::LLVMIntSLT,
            IRelationalOp::LtU => LLVMIntPredicate::LLVMIntULT,
            IRelationalOp::LeS => LLVMIntPredicate::LLVMIntSLE,
            IRelationalOp::LeU => LLVMIntPredicate::LLVMIntULE,
            IRelationalOp::GtS => LLVMIntPredicate::LLVMIntSGT,
            IRelationalOp::GtU => LLVMIntPredicate::LLVMIntUGT,
            IRelationalOp::GeS => LLVMIntPredicate::LLVMIntSGE,
            IRelationalOp::GeU => LLVMIntPredicate::LLVMIntUGE,
        }
    }
}

pub(super) struct FRelationalOpConv(pub(super) FRelationalOp);
impl From<FRelationalOpConv> for LLVMRealPredicate {
    fn from(val: FRelationalOpConv) -> Self {
        match val.0 {
            FRelationalOp::Eq => LLVMRealPredicate::LLVMRealOEQ,
            FRelationalOp::Ne => LLVMRealPredicate::LLVMRealUNE,
            FRelationalOp::Lt => LLVMRealPredicate::LLVMRealOLT,
            FRelationalOp::Le => LLVMRealPredicate::LLVMRealOLE,
            FRelationalOp::Gt => LLVMRealPredicate::LLVMRealOGT,
            FRelationalOp::Ge => LLVMRealPredicate::LLVMRealOGE,
        }
    }
}

impl Translator<'_> {
    pub(crate) fn translate_numeric(
        &self,
        instr_type: NumericInstructionCategory,
        instruction: InstructionType,
        decoder: &mut InstructionDecoder,
        variable_map: &mut [LLVMValueRef],
        llvm_function: &Function,
    ) -> Result<(), TranslationError> {
        match instr_type {
            NumericInstructionCategory::IBinary(_) => {
                let instr = decoder.read::<IBinaryInstruction>(instruction)?;
                let lhs = variable_map[instr.lhs];
                let rhs = variable_map[instr.rhs];
                let out = &mut variable_map[instr.out1];
                *out = self.compile_ibinary(instr, lhs, rhs, llvm_function)?;
            }
            NumericInstructionCategory::FBinary(_) => {
                let instr = decoder.read::<FBinaryInstruction>(instruction)?;
                let lhs = variable_map[instr.lhs];
                let rhs = variable_map[instr.rhs];
                let out = &mut variable_map[instr.out1];
                *out = self.compile_fbinary(instr, lhs, rhs)?;
            }
            NumericInstructionCategory::IUnary(_) => {
                let instr = decoder.read::<IUnaryInstruction>(instruction)?;
                let src = variable_map[instr.in1];
                let dst = &mut variable_map[instr.out1];
                *dst = self.compile_iunary(instr, src)?;
            }
            NumericInstructionCategory::FUnary(_) => {
                let instr = decoder.read::<FUnaryInstruction>(instruction)?;
                let src = variable_map[instr.in1];
                let dst = &mut variable_map[instr.out1];
                *dst = self.compile_funary(instr, src)?;
            }
            NumericInstructionCategory::Conversion(ConversionOp::Convert) => {
                let instr = decoder.read::<ConvertInstruction>(instruction)?;
                let src = variable_map[instr.in1];
                variable_map[instr.out1] = self.builder.build_int2float(
                    src,
                    self.builder.valtype2llvm(ValType::Number(instr.out1_type)),
                    instr.signed,
                    "convert",
                );
            }
            NumericInstructionCategory::Conversion(ConversionOp::Reinterpret) => {
                let instr = decoder.read::<ReinterpretInstruction>(instruction)?;
                let src = variable_map[instr.in1];
                variable_map[instr.out1] = self.builder.build_bitcast(
                    src,
                    self.builder.valtype2llvm(ValType::Number(instr.out1_type)),
                    "reinterpret",
                );
            }
            NumericInstructionCategory::Conversion(ConversionOp::Demote) => {
                let instr = decoder.read::<DemoteInstruction>(instruction)?;
                let src = variable_map[instr.in1];
                variable_map[instr.out1] =
                    self.builder
                        .build_float_cast(src, self.builder.f32(), "demote");
            }
            NumericInstructionCategory::Conversion(ConversionOp::ExtendBits) => {
                let instr = decoder.read::<ExtendBitsInstruction>(instruction)?;
                let src = variable_map[instr.in1];
                let actual_type = self.builder.custom_type(instr.input_size as u32);
                let part_view = self
                    .builder
                    .build_int_cast(src, actual_type, false, "downcast");
                variable_map[instr.out1] = self.builder.build_int_cast(
                    part_view,
                    self.builder.valtype2llvm(ValType::Number(instr.out1_type)),
                    true,
                    "extendbits",
                );
            }
            NumericInstructionCategory::Conversion(ConversionOp::ExtendType) => {
                let instr = decoder.read::<ExtendTypeInstruction>(instruction)?;
                let src = variable_map[instr.in1];
                variable_map[instr.out1] =
                    self.builder
                        .build_int_cast(src, self.builder.i64(), instr.signed, "convert");
            }
            NumericInstructionCategory::Conversion(ConversionOp::Promote) => {
                let instr = decoder.read::<PromoteInstruction>(instruction)?;
                let src = variable_map[instr.in1];
                variable_map[instr.out1] = self.builder.call_funary_constrained_intrinsic(
                    NumType::F32,
                    NumType::F64,
                    src,
                    "llvm.experimental.constrained.fpext",
                )
            }
            NumericInstructionCategory::Conversion(ConversionOp::Trunc) => {
                let instr = decoder.read::<TruncInstruction>(instruction)?;
                let src = variable_map[instr.in1];
                variable_map[instr.out1] = self.builder.build_fp2int_trunc(
                    src,
                    instr.out1_type,
                    instr.in1_type,
                    instr.signed,
                    "trunc",
                    llvm_function.get(),
                );
            }
            NumericInstructionCategory::Conversion(ConversionOp::TruncSat) => {
                let instr = decoder.read::<TruncSaturationInstruction>(instruction)?;
                let src = variable_map[instr.in1];
                let intrinsic_name = if instr.signed {
                    "llvm.fptosi.sat"
                } else {
                    "llvm.fptoui.sat"
                };
                variable_map[instr.out1] = self.builder.call_unary_intrinsic(
                    instr.in1_type,
                    instr.out1_type,
                    src,
                    intrinsic_name,
                    true,
                )
            }
            NumericInstructionCategory::Conversion(ConversionOp::Wrap) => {
                let instr = decoder.read::<WrapInstruction>(instruction)?;
                let src = variable_map[instr.in1];
                variable_map[instr.out1] =
                    self.builder
                        .build_int_cast(src, self.builder.i32(), false, "wrap");
            }
            NumericInstructionCategory::Constant => {
                let instr = decoder.read::<Constant>(instruction)?;
                variable_map[instr.out1] = match instr.out1_type {
                    NumType::I32 => self.builder.const_i32(instr.imm.into()),
                    NumType::I64 => self.builder.const_i64(instr.imm.into()),
                    NumType::F32 => self.builder.const_f32(instr.imm.into()),
                    NumType::F64 => self.builder.const_f64(instr.imm.into()),
                };
            }
            NumericInstructionCategory::IRelational(_) => {
                let instr = decoder.read::<IRelationalInstruction>(instruction)?;
                let lhs = variable_map[instr.in1];
                let rhs = variable_map[instr.in2];
                let out = &mut variable_map[instr.out1];
                *out =
                    self.builder
                        .build_icmp(IRelationalOpConv(instr.op).into(), lhs, rhs, "icmp");
                *out = self.builder.build_int_cast(
                    *out,
                    self.builder.i32(),
                    false,
                    "upcast icmp i8 -> i32",
                )
            }
            NumericInstructionCategory::FRelational(_) => {
                let instr = decoder.read::<FRelationalInstruction>(instruction)?;
                let lhs = variable_map[instr.in1];
                let rhs = variable_map[instr.in2];
                let out = &mut variable_map[instr.out1];
                *out =
                    self.builder
                        .build_fcmp(FRelationalOpConv(instr.op).into(), lhs, rhs, "fcmp");
                *out = self.builder.build_int_cast(
                    *out,
                    self.builder.i32(),
                    false,
                    "upcast fcmp i8 -> i32",
                )
            }
            NumericInstructionCategory::ITest(ITestOp::Eqz) => {
                let instr = decoder.read::<ITestInstruction>(instruction)?;
                let val = variable_map[instr.in1];
                let out = &mut variable_map[instr.out1];
                *out = self.builder.build_icmp(
                    LLVMIntPredicate::LLVMIntEQ,
                    val,
                    self.builder.const_zero(ValType::Number(instr.input_type)),
                    "test_eqz",
                );
                *out = self
                    .builder
                    .build_int_cast(*out, self.builder.i32(), false, "icmp -> i32")
            }
        }
        Ok(())
    }

    pub(crate) fn compile_ibinary(
        &self,
        instr: IBinaryInstruction,
        lhs: LLVMValueRef,
        rhs: LLVMValueRef,
        llvm_function: &Function,
    ) -> Result<LLVMValueRef, TranslationError> {
        Ok(match instr.op {
            IBinaryOp::Add => self.builder.build_add(lhs, rhs, "add"),
            IBinaryOp::Sub => self.builder.build_sub(lhs, rhs, "sub"),
            IBinaryOp::Mul => self.builder.build_mul(lhs, rhs, "mul"),
            IBinaryOp::DivU => self
                .builder
                .build_udiv(lhs, rhs, "udiv", llvm_function.get()),
            IBinaryOp::DivS => self
                .builder
                .build_sdiv(lhs, rhs, "sdiv", llvm_function.get()),
            IBinaryOp::RemU => self
                .builder
                .build_urem(lhs, rhs, "urem", llvm_function.get()),
            IBinaryOp::RemS => self
                .builder
                .build_srem(lhs, rhs, "srem", llvm_function.get()),
            IBinaryOp::And => self.builder.build_and(lhs, rhs, "and"),
            IBinaryOp::Or => self.builder.build_or(lhs, rhs, "or"),
            IBinaryOp::Xor => self.builder.build_xor(lhs, rhs, "xor"),
            IBinaryOp::Shl => self.builder.build_shl(lhs, rhs, "shl"),
            IBinaryOp::ShrS => self.builder.build_ashr(lhs, rhs, "ashr"),
            IBinaryOp::ShrU => self.builder.build_lshr(lhs, rhs, "lshr"),
            IBinaryOp::Rotl => self.builder.build_rotl(lhs, rhs, instr.types),
            IBinaryOp::Rotr => self.builder.build_rotr(lhs, rhs, instr.types),
        })
    }

    pub(crate) fn compile_fbinary(
        &self,
        instr: FBinaryInstruction,
        lhs: LLVMValueRef,
        rhs: LLVMValueRef,
    ) -> Result<LLVMValueRef, TranslationError> {
        Ok(match instr.op {
            FBinaryOp::Add => {
                // https://llvm.org/docs/LangRef.html#llvm-experimental-constrained-fadd-intrinsic
                self.builder.call_fbinary_constrained_intrinsic(
                    instr.types,
                    [lhs, rhs],
                    "llvm.experimental.constrained.fadd",
                )
            }
            FBinaryOp::Sub => {
                // https://llvm.org/docs/LangRef.html#llvm-experimental-constrained-fsub-intrinsic
                self.builder.call_fbinary_constrained_intrinsic(
                    instr.types,
                    [lhs, rhs],
                    "llvm.experimental.constrained.fsub",
                )
            }
            FBinaryOp::Mul => {
                // https://llvm.org/docs/LangRef.html#llvm-experimental-constrained-fmul-intrinsic
                self.builder.call_fbinary_constrained_intrinsic(
                    instr.types,
                    [lhs, rhs],
                    "llvm.experimental.constrained.fmul",
                )
            }
            FBinaryOp::Div => {
                // https://llvm.org/docs/LangRef.html#llvm-experimental-constrained-fdiv-intrinsic
                self.builder.call_fbinary_constrained_intrinsic(
                    instr.types,
                    [lhs, rhs],
                    "llvm.experimental.constrained.fdiv",
                )
            }
            FBinaryOp::Min => {
                // https://llvm.org/docs/LangRef.html#llvm-minimum-intrinsic
                self.builder.call_binary_intrinsic(
                    [instr.types, instr.types],
                    [lhs, rhs],
                    instr.types,
                    "llvm.minimum",
                )
            }
            FBinaryOp::Max => {
                // https://llvm.org/docs/LangRef.html#llvm-maximum-intrinsic
                self.builder.call_binary_intrinsic(
                    [instr.types, instr.types],
                    [lhs, rhs],
                    instr.types,
                    "llvm.maximum",
                )
            }
            FBinaryOp::Copysign => {
                // https://llvm.org/docs/LangRef.html#llvm-copysign-intrinsic
                self.builder.call_binary_intrinsic(
                    [instr.types, instr.types],
                    [lhs, rhs],
                    instr.types,
                    "llvm.copysign",
                )
            }
        })
    }

    pub(crate) fn compile_iunary(
        &self,
        instr: IUnaryInstruction,
        val: LLVMValueRef,
    ) -> Result<LLVMValueRef, TranslationError> {
        Ok(match instr.op {
            // https://llvm.org/docs/LangRef.html#llvm-ctlz-intrinsic
            IUnaryOp::Clz => self.builder.call_binary_intrinsic_raw(
                [
                    self.builder.valtype2llvm(ValType::Number(instr.types)),
                    self.builder.i1(),
                ],
                [val, self.builder.const_i1(false)],
                self.builder.valtype2llvm(ValType::Number(instr.types)),
                format!("llvm.ctlz.{}", instr.types).as_str(),
            ),
            // https://llvm.org/docs/LangRef.html#llvm-cttz-intrinsic
            IUnaryOp::Ctz => self.builder.call_binary_intrinsic_raw(
                [
                    self.builder.valtype2llvm(ValType::Number(instr.types)),
                    self.builder.i1(),
                ],
                [val, self.builder.const_i1(false)],
                self.builder.valtype2llvm(ValType::Number(instr.types)),
                format!("llvm.cttz.{}", instr.types).as_str(),
            ),
            // https://llvm.org/docs/LangRef.html#llvm-ctpop-intrinsic
            IUnaryOp::Popcnt => self.builder.call_unary_intrinsic(
                instr.types,
                instr.types,
                val,
                "llvm.ctpop",
                false,
            ),
        })
    }

    pub(crate) fn compile_funary(
        &self,
        instr: FUnaryInstruction,
        val: LLVMValueRef,
    ) -> Result<LLVMValueRef, TranslationError> {
        Ok(match instr.op {
            FUnaryOp::Neg => self.builder.build_fneg(val, "fneg"),

            // https://llvm.org/docs/LangRef.html#llvm-fabs-intrinsic
            FUnaryOp::Abs => {
                self.builder
                    .call_unary_intrinsic(instr.types, instr.types, val, "llvm.fabs", false)
            }
            // https://llvm.org/docs/LangRef.html#llvm-ceil-intrinsic
            FUnaryOp::Ceil => {
                self.builder
                    .call_unary_intrinsic(instr.types, instr.types, val, "llvm.ceil", false)
            }
            // https://llvm.org/docs/LangRef.html#llvm-floor-intrinsic
            FUnaryOp::Floor => self.builder.call_unary_intrinsic(
                instr.types,
                instr.types,
                val,
                "llvm.floor",
                false,
            ),
            // https://llvm.org/docs/LangRef.html#llvm-trunc-intrinsic
            FUnaryOp::Trunc => self.builder.call_unary_intrinsic(
                instr.types,
                instr.types,
                val,
                "llvm.trunc",
                false,
            ),
            // https://llvm.org/docs/LangRef.html#llvm-nearbyint-intrinsic
            FUnaryOp::Nearest => self.builder.call_unary_intrinsic(
                instr.types,
                instr.types,
                val,
                "llvm.nearbyint",
                false,
            ),
            // https://llvm.org/docs/LangRef.html#llvm-sqrt-intrinsic
            FUnaryOp::Sqrt => {
                self.builder
                    .call_unary_intrinsic(instr.types, instr.types, val, "llvm.sqrt", false)
            }
        })
    }
}
