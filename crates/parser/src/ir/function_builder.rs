use super::context::Context;
use itertools::Itertools;
use module::{
    basic_block::{BasicBlock, BasicBlockGlue, BasicBlockID},
    instructions::{PhiNode, Variable, VariableID},
    objects::function::FunctionIR,
    InstructionConsumer, InstructionEncoder,
};
use smallvec::SmallVec;
use std::{collections::HashMap, hash::BuildHasherDefault, hash::Hasher};
use wasm_types::{FuncType, LocalIdx, ValType};

pub(crate) struct FunctionIRBuilder {
    bbs: HashMap<BasicBlockID, (BasicBlock, InstructionEncoder), BuildHasherDefault<SimpleHasher>>,
    current_bb_instrs: InstructionEncoder,
    current_bb: BasicBlockID,
    locals: Vec<ValType>,
    num_vars: usize,
    func_type: FuncType,
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

impl FunctionIRBuilder {
    pub(crate) fn new() -> FunctionIRBuilder {
        FunctionIRBuilder {
            bbs: HashMap::with_capacity_and_hasher(32, BuildHasherDefault::default()),
            current_bb_instrs: InstructionEncoder::new(),
            current_bb: u32::MAX,
            locals: Vec::new(),
            num_vars: 0,
            func_type: FuncType::default(),
        }
    }

    pub(crate) fn finalize_bbs(mut self) -> Vec<BasicBlock> {
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

    fn current_bb_get(&self) -> &BasicBlock {
        self.bbs.get(&self.current_bb).map(|(bb, _)| bb).unwrap()
    }

    fn current_bb_get_mut(&mut self) -> &mut BasicBlock {
        self.bbs
            .get_mut(&self.current_bb)
            .map(|(bb, _)| bb)
            .unwrap()
    }

    pub(crate) fn try_get_current_terminator(&self) -> Option<&BasicBlockGlue> {
        self.bbs.get(&self.current_bb).map(|(bb, _)| &bb.terminator)
    }

    pub(crate) fn finalize(mut self) -> FunctionIR {
        let mut function = FunctionIR::default();
        // TODO: profile if this is faster than a clone
        function.locals = self.locals.drain(..).collect();
        function.num_vars = self.num_vars;
        function.bbs = self.finalize_bbs();
        function
    }
}

impl FunctionBuilderInterface for FunctionIRBuilder {
    fn init(&mut self, func_type: FuncType) {
        self.func_type = func_type;
    }

    fn current_bb_input_var_ids_get(&self) -> SmallVec<[VariableID; 1]> {
        self.current_bb_get()
            .inputs
            .iter()
            .map(|phi| phi.out)
            .collect()
    }

    fn current_bb_get_else_marker_out_vars(&self) -> Option<SmallVec<[VariableID; 0]>> {
        match self.current_bb_get().terminator {
            BasicBlockGlue::ElseMarker { ref output_vars } => Some(output_vars.clone()),
            _ => None,
        }
    }

    fn begin_locals(&mut self) {
        for wasm_type in self.func_type.params_iter() {
            self.locals.push(wasm_type);
        }
    }

    fn add_local(&mut self, _: LocalIdx, local_ty: ValType) {
        self.locals.push(local_ty);
    }

    fn end_locals(&mut self) {}

    fn set_var_count(&mut self, var_count: usize) {
        self.num_vars = var_count;
    }

    fn reserve_bb_with_id(&mut self, id: BasicBlockID) -> BasicBlockID {
        debug_assert!(!self.bbs.contains_key(&id));
        self.bbs
            .insert(id, (BasicBlock::new(id), InstructionEncoder::new()));
        id
    }

    fn set_bb_phi_inputs(
        &mut self,
        id: BasicBlockID,
        ctxt: &mut Context,
        inputs: impl Iterator<Item = ValType>,
    ) {
        self.bbs.get_mut(&id).unwrap().0.inputs = inputs
            .map(|var_type| PhiNode {
                inputs: smallvec::smallvec!(),
                out: ctxt.create_var(var_type).id,
                r#type: var_type,
            })
            .collect::<SmallVec<_>>();
    }

    fn put_phi_inputs_on_stack(&mut self, ctxt: &mut Context) {
        for phi in self.current_bb_get_mut().inputs.iter() {
            ctxt.push_var(Variable {
                id: phi.out,
                type_: phi.r#type,
            });
        }
    }

    fn replace_phi_inputs_on_stack(&mut self, ctxt: &mut Context) {
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

    fn continue_bb(&mut self, id: BasicBlockID) {
        if self.current_bb != u32::MAX {
            std::mem::swap(
                &mut self.current_bb_instrs,
                &mut self.bbs.get_mut(&self.current_bb).unwrap().1,
            );
        }
        self.current_bb = id;
        std::mem::swap(
            &mut self.current_bb_instrs,
            &mut self.bbs.get_mut(&self.current_bb).unwrap().1,
        );
    }

    fn current_bb_instrs(&mut self) -> &mut dyn InstructionConsumer {
        &mut self.current_bb_instrs
    }

    fn terminate_jmp(&mut self, target: BasicBlockID, output_vars: SmallVec<[VariableID; 0]>) {
        if let Some((bb, _)) = self.bbs.get_mut(&target) {
            for (phi, bb_out_var) in bb.inputs.iter_mut().zip(output_vars.iter()) {
                phi.inputs.push((self.current_bb, *bb_out_var));
            }
        }
        self.current_bb_get_mut().terminator = BasicBlockGlue::Jmp {
            target,
            output_vars,
        };
    }

    fn terminate_else(&mut self, output_vars: SmallVec<[VariableID; 0]>) {
        self.current_bb_get_mut().terminator = BasicBlockGlue::ElseMarker { output_vars };
    }

    fn terminate_return(&mut self, return_vars: SmallVec<[VariableID; 1]>) {
        self.current_bb_get_mut().terminator = BasicBlockGlue::Return { return_vars };
    }

    fn terminate_unreachable(&mut self) {
        self.current_bb_get_mut().terminator = BasicBlockGlue::Unreachable {};
    }

    fn terminate_call_indirect(
        &mut self,
        type_idx: u32,
        selector_var: VariableID,
        table_idx: u32,
        return_bb: BasicBlockID,
        call_params: SmallVec<[VariableID; 2]>,
        return_vars: SmallVec<[VariableID; 1]>,
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

    fn terminate_call(
        &mut self,
        func_idx: u32,
        return_bb: BasicBlockID,
        call_params: SmallVec<[VariableID; 2]>,
        return_vars: SmallVec<[VariableID; 1]>,
    ) {
        self.current_bb_get_mut().terminator = BasicBlockGlue::Call {
            func_idx,
            return_bb,
            call_params,
            return_vars,
        };
    }

    fn terminate_jmp_cond(
        &mut self,
        cond_var: VariableID,
        target_if_true: BasicBlockID,
        target_if_false: BasicBlockID,
        output_vars: SmallVec<[VariableID; 0]>,
    ) {
        if let Some((bb, _)) = self.bbs.get_mut(&target_if_true) {
            for (phi, bb_out_var) in bb.inputs.iter_mut().zip(output_vars.iter()) {
                phi.inputs.push((self.current_bb, *bb_out_var));
            }
        }
        if let Some((bb, _)) = self.bbs.get_mut(&target_if_false) {
            for (phi, bb_out_var) in bb.inputs.iter_mut().zip(output_vars.iter()) {
                phi.inputs.push((self.current_bb, *bb_out_var));
            }
        }
        self.current_bb_get_mut().terminator = BasicBlockGlue::JmpCond {
            cond_var,
            target_if_true,
            target_if_false,
            output_vars,
        };
    }

    fn terminate_jmp_table(
        &mut self,
        selector_var: VariableID,
        targets: SmallVec<[BasicBlockID; 5]>,
        targets_output_vars: SmallVec<[SmallVec<[VariableID; 0]>; 8]>,
        default_target: BasicBlockID,
        default_output_vars: SmallVec<[VariableID; 0]>,
    ) {
        for (target, target_out_vars) in targets
            .iter()
            .chain([&default_target])
            .zip(targets_output_vars.iter().chain([&default_output_vars]))
            .unique()
        {
            if let Some((bb, _)) = self.bbs.get_mut(target) {
                for (phi, bb_out_var) in bb.inputs.iter_mut().zip(target_out_vars.iter()) {
                    phi.inputs.push((self.current_bb, *bb_out_var));
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

    fn current_bb_id_get(&self) -> BasicBlockID {
        self.current_bb
    }

    fn eliminate_current_bb(&mut self) {
        self.bbs.remove(&self.current_bb);
        self.current_bb = u32::MAX;
        self.current_bb_instrs = InstructionEncoder::new();
    }
}

pub trait FunctionBuilderInterface {
    fn init(&mut self, func_type: FuncType);
    fn begin_locals(&mut self);
    fn add_local(&mut self, local_idx: LocalIdx, local_ty: ValType);
    fn end_locals(&mut self);
    fn set_var_count(&mut self, var_count: usize);

    fn reserve_bb(&mut self) -> BasicBlockID {
        self.reserve_bb_with_id(module::BasicBlock::next_id())
    }

    fn reserve_bb_with_id(&mut self, id: BasicBlockID) -> BasicBlockID;
    fn set_bb_phi_inputs(
        &mut self,
        id: BasicBlockID,
        ctxt: &mut Context,
        inputs: impl Iterator<Item = ValType>,
    );
    fn put_phi_inputs_on_stack(&mut self, ctxt: &mut Context);
    fn replace_phi_inputs_on_stack(&mut self, ctxt: &mut Context);
    fn continue_bb(&mut self, id: BasicBlockID);
    fn current_bb_id_get(&self) -> BasicBlockID;

    // return output_vars of else marker terminator iff terminator is else marker
    fn current_bb_get_else_marker_out_vars(&self) -> Option<SmallVec<[VariableID; 0]>>;
    fn current_bb_input_var_ids_get(&self) -> SmallVec<[VariableID; 1]>;
    fn current_bb_instrs(&mut self) -> &mut dyn InstructionConsumer;
    fn terminate_jmp(&mut self, target: BasicBlockID, output_vars: SmallVec<[VariableID; 0]>);
    fn terminate_else(&mut self, output_vars: SmallVec<[VariableID; 0]>);
    fn terminate_return(&mut self, return_vars: SmallVec<[VariableID; 1]>);
    fn terminate_unreachable(&mut self);
    fn terminate_call_indirect(
        &mut self,
        type_idx: u32,
        selector_var: VariableID,
        table_idx: u32,
        return_bb: BasicBlockID,
        call_params: SmallVec<[VariableID; 2]>,
        return_vars: SmallVec<[VariableID; 1]>,
    );
    fn terminate_call(
        &mut self,
        func_idx: u32,
        return_bb: BasicBlockID,
        call_params: SmallVec<[VariableID; 2]>,
        return_vars: SmallVec<[VariableID; 1]>,
    );
    fn terminate_jmp_cond(
        &mut self,
        cond_var: VariableID,
        target_if_true: BasicBlockID,
        target_if_false: BasicBlockID,
        output_vars: SmallVec<[VariableID; 0]>,
    );
    fn terminate_jmp_table(
        &mut self,
        selector_var: VariableID,
        targets: SmallVec<[BasicBlockID; 5]>,
        targets_output_vars: SmallVec<[SmallVec<[VariableID; 0]>; 8]>,
        default_target: BasicBlockID,
        default_output_vars: SmallVec<[VariableID; 0]>,
    );
    fn eliminate_current_bb(&mut self);
}
