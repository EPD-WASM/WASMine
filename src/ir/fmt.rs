use self::meta::PhiNode;

use super::{
    basic_block::{BasicBlock, BasicBlockGlue},
    function::Function,
    InstructionDecoder, IR,
};
use crate::structs::module::Module;
use crate::{instructions::*, structs::global::Global};
use std::fmt::{Display, Formatter};
use wasm_types::*;

pub(crate) struct IRDisplayContext<'a> {
    pub(crate) module: &'a Module,
    pub(crate) ir: &'a IR,
}

impl<'a> Display for IRDisplayContext<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (i, function) in self.ir.functions.iter().enumerate() {
            let func = FunctionDisplayContext {
                function,
                module: self.module,
                idx: i,
            };
            writeln!(f, "{}\n", func)?;
        }
        Ok(())
    }
}
pub(crate) struct FunctionDisplayContext<'a> {
    function: &'a Function,
    module: &'a Module,
    idx: usize,
}

impl<'a> Display for FunctionDisplayContext<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = Function::query_function_name(self.idx, self.module);

        write!(f, "define @{}", name)?;

        let function_type = self
            .module
            .function_types
            .get(self.function.type_idx as usize)
            .unwrap();
        let ret_types: Vec<String> = function_type.1.iter().map(|t| format!("{}", t)).collect();
        let ret_types_str = ret_types.join(", ");
        write!(f, " {}", ret_types_str)?;

        let input_types: Vec<String> = self
            .function
            .locals
            .iter()
            .take(function_type.0.len())
            .map(|t| format!("{} %{}", t.type_, t.id))
            .collect();
        let input_types_str = input_types.join(", ");
        writeln!(f, " ({}) {{", input_types_str)?;
        if self.function.import {
            write!(f, "/* imported */")?;
        } else {
            for bb in self.function.basic_blocks.iter() {
                writeln!(f, "bb{}:", bb.id)?;
                let bb = BasicBlockDisplayContext {
                    module: self.module,
                    bb,
                };
                write!(f, "{}", bb)?;
            }
        }
        write!(f, "}}")
    }
}

pub(crate) struct BasicBlockDisplayContext<'a> {
    module: &'a Module,
    bb: &'a BasicBlock,
}

#[rustfmt::skip]
impl<'a> Display for BasicBlockDisplayContext<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut decoder = InstructionDecoder::new(self.bb.instructions.clone());
        while let Ok(next_instr_t) = decoder.read_instruction_type() {
            write!(f, "\t")?;

            match next_instr_t.clone() {
                InstructionType::Memory(MemoryInstructionCategory::Load(_)) => writeln!(f, "{}", decoder.read::<LoadInstruction>(next_instr_t).unwrap())?,
                InstructionType::Memory(MemoryInstructionCategory::Store(_)) => writeln!(f, "{}", decoder.read::<StoreInstruction>(next_instr_t).unwrap())?,
                InstructionType::Memory(MemoryInstructionCategory::Memory(MemoryOp::Copy)) => writeln!(f, "{}", decoder.read::<MemoryCopyInstruction>(next_instr_t).unwrap())?,
                InstructionType::Memory(MemoryInstructionCategory::Memory(MemoryOp::Size)) => writeln!(f, "{}", decoder.read::<MemorySizeInstruction>(next_instr_t).unwrap())?,
                InstructionType::Memory(MemoryInstructionCategory::Memory(MemoryOp::Drop)) => writeln!(f, "{}", decoder.read::<DataDropInstruction>(next_instr_t).unwrap())?,
                InstructionType::Memory(MemoryInstructionCategory::Memory(MemoryOp::Fill)) => writeln!(f, "{}", decoder.read::<MemoryFillInstruction>(next_instr_t).unwrap())?,
                InstructionType::Memory(MemoryInstructionCategory::Memory(MemoryOp::Grow)) => writeln!(f, "{}", decoder.read::<MemoryGrowInstruction>(next_instr_t).unwrap())?,
                InstructionType::Memory(MemoryInstructionCategory::Memory(MemoryOp::Init)) => writeln!(f, "{}", decoder.read::<MemoryInitInstruction>(next_instr_t).unwrap())?,

                InstructionType::Numeric(NumericInstructionCategory::Constant) => writeln!(f, "{}", decoder.read::<Constant>(next_instr_t).unwrap())?,
                InstructionType::Numeric(NumericInstructionCategory::IUnary(_)) => writeln!(f, "{}", decoder.read::<IUnaryInstruction>(next_instr_t).unwrap())?,
                InstructionType::Numeric(NumericInstructionCategory::FUnary(_)) => writeln!(f, "{}", decoder.read::<FUnaryInstruction>(next_instr_t).unwrap())?,
                InstructionType::Numeric(NumericInstructionCategory::IBinary(_)) => writeln!(f, "{}", decoder.read::<IBinaryInstruction>(next_instr_t).unwrap())?,
                InstructionType::Numeric(NumericInstructionCategory::FBinary(_)) => writeln!(f, "{}", decoder.read::<FBinaryInstruction>(next_instr_t).unwrap())?,
                InstructionType::Numeric(NumericInstructionCategory::IRelational(_)) => writeln!(f, "{}", decoder.read::<IRelationalInstruction>(next_instr_t).unwrap())?,
                InstructionType::Numeric(NumericInstructionCategory::FRelational(_)) => writeln!(f, "{}", decoder.read::<FRelationalInstruction>(next_instr_t).unwrap())?,
                InstructionType::Numeric(NumericInstructionCategory::Conversion(ConversionOp::Convert)) => writeln!(f, "{}", decoder.read::<ConvertInstruction>(next_instr_t).unwrap())?,
                InstructionType::Numeric(NumericInstructionCategory::Conversion(ConversionOp::Demote)) => writeln!(f, "{}", decoder.read::<DemoteInstruction>(next_instr_t).unwrap())?,
                InstructionType::Numeric(NumericInstructionCategory::Conversion(ConversionOp::Extend)) => writeln!(f, "{}", decoder.read::<ExtendInstruction>(next_instr_t).unwrap())?,
                InstructionType::Numeric(NumericInstructionCategory::Conversion(ConversionOp::Promote)) => writeln!(f, "{}", decoder.read::<PromoteInstruction>(next_instr_t).unwrap())?,
                InstructionType::Numeric(NumericInstructionCategory::Conversion(ConversionOp::Reinterpret)) => writeln!(f, "{}", decoder.read::<ReinterpretInstruction>(next_instr_t).unwrap())?,
                InstructionType::Numeric(NumericInstructionCategory::Conversion(ConversionOp::Trunc)) => writeln!(f, "{}", decoder.read::<TruncInstruction>(next_instr_t).unwrap())?,
                InstructionType::Numeric(NumericInstructionCategory::Conversion(ConversionOp::Wrap)) => writeln!(f, "{}", decoder.read::<WrapInstruction>(next_instr_t).unwrap())?,
                InstructionType::Numeric(NumericInstructionCategory::ITest(_)) => writeln!(f, "{}", decoder.read::<ITestInstruction>(next_instr_t).unwrap())?,

                InstructionType::Parametric(ParametricInstructionType::Drop) => writeln!(f, "{}", decoder.read::<DropInstruction>(next_instr_t).unwrap())?,
                InstructionType::Parametric(ParametricInstructionType::Select) => writeln!(f, "{}", decoder.read::<SelectInstruction>(next_instr_t).unwrap())?,

                InstructionType::Table(TableInstructionCategory::Copy) => writeln!(f, "{}", decoder.read::<TableCopyInstruction>(next_instr_t).unwrap())?,
                InstructionType::Table(TableInstructionCategory::Fill) => writeln!(f, "{}", decoder.read::<TableFillInstruction>(next_instr_t).unwrap())?,
                InstructionType::Table(TableInstructionCategory::Get) => writeln!(f, "{}", decoder.read::<TableGetInstruction>(next_instr_t).unwrap())?,
                InstructionType::Table(TableInstructionCategory::Grow) => writeln!(f, "{}", decoder.read::<TableGrowInstruction>(next_instr_t).unwrap())?,
                InstructionType::Table(TableInstructionCategory::Init) => writeln!(f, "{}", decoder.read::<TableInitInstruction>(next_instr_t).unwrap())?,
                InstructionType::Table(TableInstructionCategory::Set) => writeln!(f, "{}", decoder.read::<TableSetInstruction>(next_instr_t).unwrap())?,
                InstructionType::Table(TableInstructionCategory::Size) => writeln!(f, "{}", decoder.read::<TableSizeInstruction>(next_instr_t).unwrap())?,
                InstructionType::Table(TableInstructionCategory::Drop) => writeln!(f, "{}", decoder.read::<ElemDropInstruction>(next_instr_t).unwrap())?,

                InstructionType::Reference(ReferenceInstructionType::RefFunc) => writeln!(f, "{}", decoder.read::<ReferenceFunctionInstruction>(next_instr_t).unwrap())?,
                InstructionType::Reference(ReferenceInstructionType::RefNull) => writeln!(f, "{}", decoder.read::<ReferenceNullInstruction>(next_instr_t).unwrap())?,
                InstructionType::Reference(ReferenceInstructionType::RefIsNull) => writeln!(f, "{}", decoder.read::<ReferenceIsNullInstruction>(next_instr_t).unwrap())?,

                InstructionType::Variable(VariableInstructionType::GlobalGet) => writeln!(f, "{}", decoder.read::<GlobalGetInstruction>(next_instr_t).unwrap())?,
                InstructionType::Variable(VariableInstructionType::GlobalSet) => writeln!(f, "{}", decoder.read::<GlobalSetInstruction>(next_instr_t).unwrap())?,
                InstructionType::Variable(VariableInstructionType::LocalGet) => writeln!(f, "{}", decoder.read::<LocalGetInstruction>(next_instr_t).unwrap())?,
                InstructionType::Variable(VariableInstructionType::LocalSet) => writeln!(f, "{}", decoder.read::<LocalSetInstruction>(next_instr_t).unwrap())?,
                InstructionType::Variable(VariableInstructionType::LocalTee) => writeln!(f, "{}", decoder.read::<TeeLocalInstruction>(next_instr_t).unwrap())?,

                InstructionType::Meta(MetaInstructionType::PhiNode) => writeln!(f, "{}", decoder.read::<PhiNode>(next_instr_t).unwrap())?,

                InstructionType::Control(_) => unreachable!("Control instructions should already be converted into BasicBlockGlue"),
                InstructionType::Vector(_) => unimplemented!(),
            }
        }
        writeln!(
            f,
            "{}",
            BasicBlockGlueDisplayContext {
                bbg: &self.bb.terminator,
                module: self.module
            }
        )?;
        Ok(())
    }
}

struct BasicBlockGlueDisplayContext<'a> {
    module: &'a Module,
    bbg: &'a BasicBlockGlue,
}

fn format_vars(vars: &[u32]) -> String {
    let vars: Vec<String> = vars.iter().map(|v| format!("%{}", v)).collect();
    vars.join(", ")
}

impl Display for BasicBlockGlueDisplayContext<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "\t")?;
        match self.bbg {
            BasicBlockGlue::Jmp {
                target,
                output_vars,
            } => write!(f, "jmp -> bb{}({})", target, format_vars(output_vars)),
            BasicBlockGlue::JmpCond {
                cond_var,
                target_if_true,
                target_if_false,
                output_vars,
            } => write!(
                f,
                "jmp (%{} == 0 ? bb{} : bb{})({})",
                cond_var,
                target_if_true,
                target_if_false,
                format_vars(output_vars)
            ),
            BasicBlockGlue::JmpTable {
                cond_var,
                targets,
                targets_output_vars,
                default_target,
                default_output_vars,
            } => {
                let targets = targets
                    .iter()
                    .zip(targets_output_vars.iter())
                    .map(|(target, out_vars)| format!("%{}({})", target, format_vars(out_vars)))
                    .collect::<Vec<String>>()
                    .join(", ");
                write!(
                    f,
                    "jmp_table %{} => [{}] / bb{}({})",
                    cond_var,
                    targets,
                    default_target,
                    format_vars(default_output_vars)
                )
            }
            BasicBlockGlue::Call {
                func_idx,
                return_bb,
                call_params,
            } => {
                let function_name = Function::query_function_name(*func_idx as usize, self.module);
                write!(
                    f,
                    "call {}({}) -> bb{}",
                    function_name,
                    format_vars(call_params),
                    return_bb
                )
            }
            BasicBlockGlue::CallIndirect {
                selector_var,
                table_idx,
                return_bb,
                call_params,
                ..
            } => write!(
                f,
                "call_indirect (table_{}[%{}])({}) -> bb{}",
                table_idx,
                selector_var,
                format_vars(call_params),
                return_bb
            ),
            BasicBlockGlue::Return { return_vars } => {
                write!(f, "ret {}", format_vars(return_vars))
            }
            BasicBlockGlue::ElseMarker { .. } => {
                unreachable!("ElseMarker should not be printed")
            }
            BasicBlockGlue::Unreachable => write!(f, "unreachable"),
        }
    }
}

impl Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (idx, global) in self.globals.iter().enumerate() {
            writeln!(
                f,
                "{}",
                GlobalDisplayContext {
                    global,
                    idx,
                    module: self
                }
            )?;
        }
        write!(
            f,
            "{}",
            IRDisplayContext {
                module: self,
                ir: &self.ir
            }
        )?;
        Ok(())
    }
}

struct GlobalDisplayContext<'a> {
    global: &'a Global,
    idx: usize,
    module: &'a Module,
}

impl Display for GlobalDisplayContext<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let gt = match self.global.r#type {
            GlobalType::Mut(t) => format!("mut {}", t),
            GlobalType::Const(t) => format!("const {}", t),
        };
        if self.global.import {
            write!(f, "@global_{}: {} = imported", self.idx, gt)
        } else {
            writeln!(f, "@global_{}: {} = res{{", self.idx, gt)?;
            for bb in self.global.value.instrs.iter() {
                write!(
                    f,
                    "{}",
                    BasicBlockDisplayContext {
                        bb,
                        module: self.module
                    }
                )?;
            }
            writeln!(f, "}}")
        }
    }
}
