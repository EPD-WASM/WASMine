use super::{
    basic_block::{BasicBlock, BasicBlockGlue},
    InstructionDecoder,
};
use crate::{instructions::*, objects::global::Global};
use crate::{
    objects::function::{Function, FunctionIR, FunctionImport},
    ModuleMetadata,
};
use std::fmt::{Display, Formatter};
use wasm_types::*;

pub struct IRDisplayContext<'a> {
    pub module: &'a ModuleMetadata,
}

impl<'a> Display for IRDisplayContext<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (i, function) in self.module.functions.iter().enumerate() {
            let func = FunctionDisplayContext {
                function,
                module: self.module,
                idx: i as FuncIdx,
            };
            writeln!(f, "{func}")?;
        }
        Ok(())
    }
}
pub struct FunctionDisplayContext<'a> {
    function: &'a Function,
    module: &'a ModuleMetadata,
    idx: FuncIdx,
}

impl<'a> Display for FunctionDisplayContext<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = Function::debug_function_name(self.idx, self.module);

        write!(f, "define @{name}")?;

        let type_idx = self.function.type_idx;
        let function_type = self.module.function_types.get(type_idx as usize).unwrap();
        let ret_types: Vec<String> = function_type
            .results_iter()
            .map(|t| format!("{t}"))
            .collect();
        let ret_types_str = ret_types.join(", ");
        write!(f, " {ret_types_str}")?;

        if let Some(FunctionImport { import_idx }) = self.function.get_import() {
            let import = &self.module.imports[*import_idx as usize];
            let import_name = format!("{}.{}", import.module, import.name);
            let input_types: Vec<String> = function_type
                .params_iter()
                .map(|t| format!("{t}"))
                .collect();
            let input_types_str = input_types.join(", ");
            writeln!(f, " ({input_types_str}) {{",)?;
            write!(f, "/* imported: {import_name} */",)?
        }

        if let Some(FunctionIR { bbs, locals, .. }) = self.function.get_ir() {
            let input_types: Vec<String> = locals
                .iter()
                .take(function_type.num_params())
                .map(|t| format!("{t}",))
                .collect();
            let input_types_str = input_types.join(", ");
            writeln!(f, " ({input_types_str}) {{",)?;

            for bb in bbs.iter() {
                writeln!(f, "bb{}:", bb.id)?;
                let bb = BasicBlockDisplayContext {
                    module: self.module,
                    bb,
                };
                write!(f, "{bb}",)?;
            }
        }
        write!(f, "}}")
    }
}

pub struct BasicBlockDisplayContext<'a> {
    module: &'a ModuleMetadata,
    bb: &'a BasicBlock,
}

#[rustfmt::skip]
impl<'a> Display for BasicBlockDisplayContext<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for phi in &self.bb.inputs {
            writeln!(f, "{phi}")?;
        }

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
                InstructionType::Numeric(NumericInstructionCategory::Conversion(ConversionOp::ExtendBits)) => writeln!(f, "{}", decoder.read::<ExtendBitsInstruction>(next_instr_t).unwrap())?,
                InstructionType::Numeric(NumericInstructionCategory::Conversion(ConversionOp::ExtendType)) => writeln!(f, "{}", decoder.read::<ExtendTypeInstruction>(next_instr_t).unwrap())?,
                InstructionType::Numeric(NumericInstructionCategory::Conversion(ConversionOp::Promote)) => writeln!(f, "{}", decoder.read::<PromoteInstruction>(next_instr_t).unwrap())?,
                InstructionType::Numeric(NumericInstructionCategory::Conversion(ConversionOp::Reinterpret)) => writeln!(f, "{}", decoder.read::<ReinterpretInstruction>(next_instr_t).unwrap())?,
                InstructionType::Numeric(NumericInstructionCategory::Conversion(ConversionOp::Trunc)) => writeln!(f, "{}", decoder.read::<TruncInstruction>(next_instr_t).unwrap())?,
                InstructionType::Numeric(NumericInstructionCategory::Conversion(ConversionOp::TruncSat)) => writeln!(f, "{}", decoder.read::<TruncSaturationInstruction>(next_instr_t).unwrap())?,
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
                InstructionType::Variable(VariableInstructionType::LocalTee) => writeln!(f, "{}", decoder.read::<LocalTeeInstruction>(next_instr_t).unwrap())?,

                InstructionType::Meta(MetaInstructionType::PhiNode) => unreachable!("Phi nodes are not encoded."),
                InstructionType::Control(_) => unreachable!("Control instructions should already be converted into BasicBlockGlue"),

                InstructionType::Vector => unimplemented!(),
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
    module: &'a ModuleMetadata,
    bbg: &'a BasicBlockGlue,
}

fn format_vars(vars: &[VariableID]) -> String {
    let vars: Vec<String> = vars.iter().map(|v| format!("%{v}",)).collect();
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
                target_if_false,
                target_if_true,
                format_vars(output_vars)
            ),
            BasicBlockGlue::JmpTable {
                selector_var: cond_var,
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
                return_vars,
            } => {
                let function_name = Function::debug_function_name(*func_idx, self.module);
                match return_vars.len() {
                    0 => write!(f, "void = ")?,
                    1 => write!(f, "%{} = ", return_vars[0])?,
                    _ => write!(f, "({}) = ", format_vars(return_vars))?,
                }
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
                return_vars,
                ..
            } => {
                match return_vars.len() {
                    0 => write!(f, "void = ")?,
                    1 => write!(f, "%{} = ", return_vars[0])?,
                    _ => write!(f, "({}) = ", format_vars(return_vars))?,
                }
                write!(
                    f,
                    "call_indirect (table_{}[%{}])({}) -> bb{}",
                    table_idx,
                    selector_var,
                    format_vars(call_params),
                    return_bb
                )
            }
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

impl Display for ModuleMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (idx, global) in self.globals.iter().enumerate() {
            writeln!(f, "{}", GlobalDisplayContext { global, idx })?;
        }
        write!(f, "{}", IRDisplayContext { module: self })?;
        Ok(())
    }
}

struct GlobalDisplayContext<'a> {
    global: &'a Global,
    idx: usize,
}

impl Display for GlobalDisplayContext<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let gt = match self.global.r#type {
            GlobalType::Mut(t) => format!("mut {t}"),
            GlobalType::Const(t) => format!("const {t}"),
        };
        if self.global.import {
            writeln!(f, "@__wasmine_global__{}: {} = imported", self.idx, gt)
        } else {
            writeln!(
                f,
                "@__wasmine_global__{}: {} = {:?};",
                self.idx, gt, self.global.init
            )
        }
    }
}
