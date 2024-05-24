use crate::{instructions::VariableID, structs::instruction::ControlInstruction};
use lazy_static::lazy_static;
use std::{
    collections::VecDeque,
    fmt::{Debug, Formatter},
    sync::atomic::{AtomicU32, Ordering},
};
use wasm_types::*;

lazy_static! {
    pub(crate) static ref BASIC_BLOCK_ID: AtomicU32 = AtomicU32::new(0);
}

pub(crate) type BasicBlockID = u32;

#[derive(Default, Clone, Debug)]
pub(crate) struct BasicBlock {
    // instructions encoded
    pub(crate) instructions: BasicBlockStorage,
    pub(crate) terminator: BasicBlockGlue,
    pub(crate) id: BasicBlockID,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct BasicBlockStorage {
    pub(crate) immediate_storage: VecDeque<u8>,
    pub(crate) variable_storage: VecDeque<VariableID>,
    pub(crate) type_storage: VecDeque<ValType>,
    pub(crate) instruction_storage: VecDeque<InstructionType>,
    pub(crate) terminator: ControlInstruction,
}

impl BasicBlock {
    pub(crate) fn new(id: u32) -> BasicBlock {
        BasicBlock {
            id,
            ..Default::default()
        }
    }

    pub(crate) fn next_id() -> u32 {
        BASIC_BLOCK_ID.fetch_add(1, Ordering::SeqCst)
    }

    pub(crate) fn successors(&self) -> impl Iterator<Item = BasicBlockID> {
        match &self.terminator {
            BasicBlockGlue::Jmp { target, .. } => vec![*target].into_iter(),
            BasicBlockGlue::JmpCond {
                target_if_true,
                target_if_false,
                ..
            } => vec![*target_if_true, *target_if_false].into_iter(),
            BasicBlockGlue::JmpTable {
                targets,
                default_target,
                ..
            } => {
                let mut res = targets.clone();
                res.push(*default_target);
                res.into_iter()
            }
            _ => vec![].into_iter(),
        }
    }

    pub(crate) fn target_out_vars(&self, target: BasicBlockID) -> impl Iterator<Item = VariableID> {
        match &self.terminator {
            BasicBlockGlue::Jmp {
                output_vars,
                target: jmp_target,
            } => {
                debug_assert_eq!(target, *jmp_target);
                output_vars.clone().into_iter()
            }
            BasicBlockGlue::JmpCond {
                output_vars,
                target_if_true,
                target_if_false,
                ..
            } => {
                debug_assert!(target == *target_if_true || target == *target_if_false);
                output_vars.clone().into_iter()
            }
            BasicBlockGlue::JmpTable {
                default_output_vars,
                targets_output_vars,
                targets,
                default_target,
                ..
            } => {
                debug_assert!(target == *default_target || targets.contains(&target));
                if target == *default_target {
                    default_output_vars.clone().into_iter()
                } else {
                    let idx = targets.iter().position(|&x| x == target).unwrap();
                    targets_output_vars[idx].clone().into_iter()
                }
            }
            _ => vec![].into_iter(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) enum BasicBlockGlue {
    // jump to another block
    Jmp {
        target: BasicBlockID,
        output_vars: Vec<VariableID>,
    },

    JmpCond {
        cond_var: VariableID,
        target_if_true: BasicBlockID,
        target_if_false: BasicBlockID,
        output_vars: Vec<VariableID>,
    },

    JmpTable {
        cond_var: VariableID,
        targets: Vec<BasicBlockID>,
        targets_output_vars: Vec<Vec<VariableID>>,
        default_target: BasicBlockID,
        default_output_vars: Vec<VariableID>,
    },

    Call {
        func_idx: u32,
        return_bb: BasicBlockID,
        call_params: Vec<VariableID>,
    },

    CallIndirect {
        type_idx: u32,
        selector_var: VariableID,
        table_idx: BasicBlockID,
        return_bb: BasicBlockID,
        call_params: Vec<VariableID>,
    },

    Return {
        return_vars: Vec<VariableID>,
    },

    // only required during parsing
    ElseMarker {
        output_vars: Vec<VariableID>,
    },

    #[default]
    Unreachable,
}
