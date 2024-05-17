use crate::instructions::{storage::InstructionStorage, VariableID};
use lazy_static::lazy_static;
use std::{
    fmt::{Debug, Formatter},
    sync::atomic::{AtomicU32, Ordering},
};

lazy_static! {
    pub(crate) static ref BASIC_BLOCK_ID: AtomicU32 = AtomicU32::new(0);
}

pub(crate) type BasicBlockID = u32;

#[derive(Default, Clone)]
pub(crate) struct BasicBlock {
    // instructions encoded
    pub(crate) instructions: InstructionStorage,
    pub(crate) terminator: BasicBlockGlue,
    pub(crate) id: BasicBlockID,
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
}

impl Debug for BasicBlock {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "BasicBlock {{ id: {}, terminator: {:?}, instructions: {} }}",
            self.id, self.terminator, self.instructions
        )
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
