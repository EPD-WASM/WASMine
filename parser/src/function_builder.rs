use crate::context::Context;
use ir::{
    basic_block::{BasicBlock, BasicBlockGlue, BasicBlockID},
    instructions::{PhiNode, Variable},
    structs::instruction::ControlInstruction,
    InstructionEncoder,
};
use itertools::Itertools;
use std::{collections::HashMap, hash::BuildHasherDefault, hash::Hasher};
use wasm_types::ValType;

pub(crate) struct FunctionBuilder {
    bbs: HashMap<BasicBlockID, (BasicBlock, InstructionEncoder), BuildHasherDefault<SimpleHasher>>,
    current_bb_instrs: InstructionEncoder,
    current_bb: BasicBlockID,
}

#[derive(Default)]
pub struct SimpleHasher(u32);

impl Hasher for SimpleHasher {
    fn finish(&self) -> u64 {
        self.0 as u64
    }

    fn write(&mut self, bytes: &[u8]) {
        self.0 = u32::from_le_bytes(bytes.try_into().unwrap());
    }
}

impl FunctionBuilder {
    pub(crate) fn new() -> FunctionBuilder {
        FunctionBuilder {
            bbs: HashMap::with_capacity_and_hasher(32, BuildHasherDefault::default()),
            current_bb_instrs: InstructionEncoder::new(),
            current_bb: u32::MAX,
        }
    }

    pub(crate) fn start_bb(&mut self) {
        self.start_bb_with_id(BasicBlock::next_id());
    }

    pub(crate) fn start_bb_with_id(&mut self, id: BasicBlockID) {
        debug_assert!(!self.bbs.contains_key(&id));
        if self.current_bb != u32::MAX {
            std::mem::swap(
                &mut self.bbs.get_mut(&self.current_bb).unwrap().1,
                &mut self.current_bb_instrs,
            );
        }
        self.bbs
            .insert(id, (BasicBlock::new(id), InstructionEncoder::new()));
        self.current_bb_instrs = InstructionEncoder::new();
        self.current_bb = id;
    }

    pub(crate) fn reserve_bb_with_phis(
        &mut self,
        id: BasicBlockID,
        ctxt: &mut Context,
        inputs: impl Iterator<Item = ValType>,
    ) {
        self.start_bb_with_id(id);
        self.bbs.get_mut(&id).unwrap().0.inputs = inputs
            .map(|var_type| PhiNode {
                inputs: Vec::new(),
                out: ctxt.create_var(var_type).id,
                r#type: var_type,
            })
            .collect::<Vec<_>>();
    }

    pub(crate) fn put_phi_inputs_on_stack(&mut self, ctxt: &mut Context) {
        for phi in self.current_bb_get_mut().inputs.iter_mut() {
            ctxt.push_var(Variable {
                id: phi.out,
                type_: phi.r#type,
            });
        }
    }

    pub(crate) fn replace_phi_inputs_on_stack(&mut self, ctxt: &mut Context) {
        for _ in self.current_bb_get_mut().inputs.iter() {
            ctxt.pop_var();
        }
        for phi in self.current_bb_get_mut().inputs.iter_mut() {
            ctxt.push_var(Variable {
                id: phi.out,
                type_: phi.r#type,
            });
        }
    }

    pub(crate) fn continue_bb(&mut self, id: BasicBlockID) {
        std::mem::swap(
            &mut self.current_bb_instrs,
            &mut self.bbs.get_mut(&self.current_bb).unwrap().1,
        );
        self.current_bb = id;
        std::mem::swap(
            &mut self.current_bb_instrs,
            &mut self.bbs.get_mut(&self.current_bb).unwrap().1,
        );
    }

    pub(crate) fn current_bb_try_get(&self) -> Option<&BasicBlock> {
        self.bbs.get(&self.current_bb).map(|(bb, _)| bb)
    }

    pub(crate) fn current_bb_get(&self) -> &BasicBlock {
        self.bbs.get(&self.current_bb).map(|(bb, _)| bb).unwrap()
    }

    pub(crate) fn current_bb_get_mut(&mut self) -> &mut BasicBlock {
        self.bbs
            .get_mut(&self.current_bb)
            .map(|(bb, _)| bb)
            .unwrap()
    }

    pub(crate) fn current_bb_instrs(&mut self) -> &mut InstructionEncoder {
        &mut self.current_bb_instrs
    }

    pub(crate) fn current_bb_terminator(&self) -> &ControlInstruction {
        self.current_bb_instrs.peek_terminator()
    }

    pub(crate) fn terminate_jmp(&mut self, target: BasicBlockID, output_vars: Vec<u32>) {
        if let Some((bb, _)) = self.bbs.get_mut(&target) {
            for (phi, bb_out_var) in bb.inputs.iter_mut().zip(output_vars.iter()) {
                phi.inputs
                    .push((self.current_bb as BasicBlockID, *bb_out_var));
            }
        }
        self.current_bb_get_mut().terminator = BasicBlockGlue::Jmp {
            target,
            output_vars,
        };
    }

    pub(crate) fn terminate_else(&mut self, output_vars: Vec<u32>) {
        self.current_bb_get_mut().terminator = BasicBlockGlue::ElseMarker { output_vars };
    }

    pub(crate) fn terminate_return(&mut self, return_vars: Vec<u32>) {
        self.current_bb_get_mut().terminator = BasicBlockGlue::Return { return_vars };
    }

    pub(crate) fn terminate_unreachable(&mut self) {
        self.current_bb_get_mut().terminator = BasicBlockGlue::Unreachable {};
    }

    pub(crate) fn terminate_call_indirect(
        &mut self,
        type_idx: u32,
        selector_var: u32,
        table_idx: u32,
        return_bb: BasicBlockID,
        call_params: Vec<u32>,
        return_vars: Vec<u32>,
    ) {
        self.current_bb_get_mut().terminator = BasicBlockGlue::CallIndirect {
            type_idx,
            selector_var,
            table_idx,
            return_bb,
            call_params,
            return_vars,
        };
    }

    pub(crate) fn terminate_call(
        &mut self,
        func_idx: u32,
        return_bb: BasicBlockID,
        call_params: Vec<u32>,
        return_vars: Vec<u32>,
    ) {
        self.current_bb_get_mut().terminator = BasicBlockGlue::Call {
            func_idx,
            return_bb,
            call_params,
            return_vars,
        };
    }

    pub(crate) fn terminate_jmp_cond(
        &mut self,
        cond_var: u32,
        target_if_true: BasicBlockID,
        target_if_false: BasicBlockID,
        output_vars: Vec<u32>,
    ) {
        if let Some((bb, _)) = self.bbs.get_mut(&target_if_true) {
            for (phi, bb_out_var) in bb.inputs.iter_mut().zip(output_vars.iter()) {
                phi.inputs
                    .push((self.current_bb as BasicBlockID, *bb_out_var));
            }
        }
        if let Some((bb, _)) = self.bbs.get_mut(&target_if_false) {
            for (phi, bb_out_var) in bb.inputs.iter_mut().zip(output_vars.iter()) {
                phi.inputs
                    .push((self.current_bb as BasicBlockID, *bb_out_var));
            }
        }
        self.current_bb_get_mut().terminator = BasicBlockGlue::JmpCond {
            cond_var,
            target_if_true,
            target_if_false,
            output_vars,
        };
    }

    pub(crate) fn terminate_jmp_table(
        &mut self,
        selector_var: u32,
        targets: Vec<BasicBlockID>,
        targets_output_vars: Vec<Vec<u32>>,
        default_target: BasicBlockID,
        default_output_vars: Vec<u32>,
    ) {
        for (target, target_out_vars) in targets
            .iter()
            .chain([&default_target])
            .zip(targets_output_vars.iter().chain([&default_output_vars]))
            .unique()
        {
            if let Some((bb, _)) = self.bbs.get_mut(target) {
                for (phi, bb_out_var) in bb.inputs.iter_mut().zip(target_out_vars.iter()) {
                    phi.inputs
                        .push((self.current_bb as BasicBlockID, *bb_out_var));
                }
            }
        }

        self.current_bb_get_mut().terminator = BasicBlockGlue::JmpTable {
            selector_var,
            targets,
            targets_output_vars,
            default_target,
            default_output_vars,
        };
    }

    pub(crate) fn eliminate_current_bb(&mut self) {
        self.bbs.remove(&self.current_bb);
        self.current_bb = u32::MAX;
        self.current_bb_instrs = InstructionEncoder::new();
    }

    pub(crate) fn finalize(mut self) -> Vec<BasicBlock> {
        std::mem::swap(
            &mut self.bbs.get_mut(&self.current_bb).unwrap().1,
            &mut self.current_bb_instrs,
        );

        self.bbs
            .into_iter()
            .sorted_by_key(|(id, _)| *id)
            .map(|(_, (mut bb, instrs))| {
                bb.instructions = instrs.extract_data();
                bb
            })
            .collect()
    }
}
