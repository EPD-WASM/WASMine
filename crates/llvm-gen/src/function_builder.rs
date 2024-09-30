use crate::{
    abstraction::{builder::Builder, function::Function, module::Module},
    instructions::instruction_consumer::LLVMInstructionConsumer,
    util::c_str,
    Context, Translator,
};
use llvm_sys::{
    core::LLVMBuildExtractValue,
    prelude::{LLVMBasicBlockRef, LLVMTypeRef, LLVMValueRef},
};
use module::{
    instructions::{Variable, VariableID},
    BasicBlockID,
};
use parser::FunctionBuilderInterface;
use smallvec::SmallVec;
use std::{cell::RefCell, collections::HashMap, rc::Rc, u32};
use wasm_types::{FuncIdx, FuncType, LocalIdx, ValType};

// modelling a state machine
// TODO: continue integration with functionbuilder
#[cfg(debug_assertions)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum FunctionBuilderState {
    #[default]
    Invalid,
    Initialized,
    AddingLocals,
    AfterLocals,
}

struct CurrentBlockCtxt<'wasm> {
    bb: LLVMBasicBlockRef,
    phi_inputs: Vec<Variable>,
    instrs: LLVMInstructionConsumer<'wasm>,
    else_marker_out_vars: Option<SmallVec<[VariableID; 0]>>,
}

pub struct LLVMFunctionBuilder<'wasm> {
    context: Rc<Context>,
    func_idx: FuncIdx,
    module: Rc<Module>,
    wasm_module: &'wasm module::ModuleMetadata,
    func_type: FuncType,
    locals: Rc<RefCell<Vec<(LLVMValueRef, LLVMTypeRef)>>>,
    llvm_functions: Rc<RefCell<Vec<Function>>>,
    vars: Rc<RefCell<Vec<LLVMValueRef>>>,

    current_bb_id: BasicBlockID,
    current_instrs: LLVMInstructionConsumer<'wasm>,
    bbs: HashMap<BasicBlockID, CurrentBlockCtxt<'wasm>>,

    #[cfg(debug_assertions)]
    state: FunctionBuilderState,
}

impl<'wasm> LLVMFunctionBuilder<'wasm> {
    pub(crate) fn new(
        ctxt: Rc<Context>,
        func_idx: FuncIdx,
        module: Rc<Module>,
        llvm_functions: Rc<RefCell<Vec<Function>>>,
        wasm_module: &'wasm module::ModuleMetadata,
    ) -> Self {
        // all self-referential structures are Rc<RefCell<_>> to avoid borrowing issues
        let locals = Rc::new(RefCell::new(Vec::new()));
        let vars = Rc::new(RefCell::new(Vec::new()));

        #[allow(invalid_value)]
        LLVMFunctionBuilder {
            context: ctxt.clone(),
            locals: locals.clone(),
            func_idx,
            func_type: FuncType::default(),
            current_bb_id: u32::MAX,
            bbs: HashMap::new(),
            current_instrs: LLVMInstructionConsumer::new(
                ctxt.clone(),
                llvm_functions.clone(),
                wasm_module,
                module.clone(),
                locals.clone(),
                vars.clone(),
                func_idx as usize,
            ),
            vars,
            module,
            llvm_functions,
            wasm_module,

            #[cfg(debug_assertions)]
            state: FunctionBuilderState::Invalid,
        }
    }

    pub(crate) fn finalize(self) {
        #[cfg(debug_assertions)]
        Translator::verify_function(
            &self.module,
            &self.llvm_functions.borrow()[self.func_idx as usize],
            self.func_idx,
            &self.wasm_module,
        )
        .unwrap();

        #[cfg(debug_assertions)]
        self.module.print_to_file();
    }

    #[inline]
    fn add_phi_input_vals(&self, target: BasicBlockID, output_vars: &[VariableID]) {
        let target_phis = &self.bbs.get(&target).unwrap().phi_inputs;
        for (target_phi_var, output_var) in target_phis.iter().zip(output_vars.iter().cloned()) {
            let target_phi_val = self.vars.borrow()[target_phi_var.id];
            Builder::phi_add_incoming(
                target_phi_val,
                &mut [self.vars.borrow()[output_var]],
                &mut [self.bbs.get(&self.current_bb_id).unwrap().bb],
            );
        }
    }
}

impl FunctionBuilderInterface for LLVMFunctionBuilder<'_> {
    fn init(&mut self, func_type: FuncType) {
        #[cfg(debug_assertions)]
        {
            assert_eq!(self.state, FunctionBuilderState::Invalid);
            self.state = FunctionBuilderState::Initialized;
        }
        self.func_type = func_type;
    }

    fn begin_locals(&mut self) {
        #[cfg(debug_assertions)]
        {
            assert_eq!(self.state, FunctionBuilderState::Initialized);
            self.state = FunctionBuilderState::AddingLocals;
        }

        let bb = self.context.append_basic_block(
            self.llvm_functions.borrow()[self.func_idx as usize].get(),
            "entry",
        );
        self.current_instrs.translator.builder.position_at_end(bb);

        // add function parameters as first locals
        for (param_idx, wasm_type) in self.func_type.params_iter().enumerate() {
            let param_llvm_type = self
                .current_instrs
                .translator
                .builder
                .valtype2llvm(wasm_type);
            let param_val =
                self.llvm_functions.borrow()[self.func_idx as usize].get_param(param_idx + 1);
            let local = self
                .current_instrs
                .translator
                .builder
                .build_alloca(param_llvm_type, &format!("local{param_idx}"));
            self.current_instrs
                .translator
                .builder
                .build_store(param_val, local);
            self.locals.borrow_mut().push((local, param_llvm_type));
        }
    }

    fn add_local(&mut self, local_idx: LocalIdx, local_ty: ValType) {
        #[cfg(debug_assertions)]
        {
            assert_eq!(self.state, FunctionBuilderState::AddingLocals);
        }
        let local_llvm_type = self
            .current_instrs
            .translator
            .builder
            .valtype2llvm(local_ty);
        let local_llvm_storage = self
            .current_instrs
            .translator
            .builder
            .build_alloca(local_llvm_type, &format!("local{local_idx}"));
        self.current_instrs.translator.builder.build_store(
            self.current_instrs.translator.builder.const_zero(local_ty),
            local_llvm_storage,
        );
        self.locals
            .borrow_mut()
            .push((local_llvm_storage, local_llvm_type));
    }

    fn end_locals(&mut self) {
        #[cfg(debug_assertions)]
        {
            assert_eq!(self.state, FunctionBuilderState::AddingLocals);
            self.state = FunctionBuilderState::AfterLocals;
        }
    }

    fn set_var_count(&mut self, _: usize) {
        // noop for now
    }

    fn reserve_bb_with_id(&mut self, id: BasicBlockID) -> BasicBlockID {
        debug_assert!(!self.bbs.contains_key(&id));
        let new_bb = self.context.append_basic_block(
            self.llvm_functions.borrow()[self.func_idx as usize].get(),
            &format!("bb{id}"),
        );
        if self.bbs.is_empty() {
            // first bb is uncoditionally jumped to from local declaration
            self.current_instrs
                .translator
                .builder
                .build_unconditional_branch(new_bb);
        }

        let new_bb_ctxt = CurrentBlockCtxt {
            bb: new_bb,
            phi_inputs: Vec::new(),
            instrs: LLVMInstructionConsumer::new(
                self.context.clone(),
                self.llvm_functions.clone(),
                self.wasm_module,
                self.module.clone(),
                self.locals.clone(),
                self.vars.clone(),
                self.func_idx as usize,
            ),
            else_marker_out_vars: None,
        };
        new_bb_ctxt.instrs.set_basic_block(new_bb);
        self.bbs.insert(id, new_bb_ctxt);
        id
    }

    fn set_bb_phi_inputs(
        &mut self,
        id: module::BasicBlockID,
        ctxt: &mut parser::Context,
        inputs: impl Iterator<Item = ValType>,
    ) {
        for val_ty in inputs {
            let builder_ref = &self.bbs.get(&id).unwrap().instrs.translator.builder;
            self.vars
                .borrow_mut()
                .push(builder_ref.build_phi(builder_ref.valtype2llvm(val_ty), "phi"));
            self.bbs
                .get_mut(&id)
                .unwrap()
                .phi_inputs
                .push(ctxt.create_var(val_ty));
            debug_assert_eq!(
                self.bbs.get(&id).unwrap().phi_inputs.last().unwrap().id,
                self.vars.borrow().len() - 1
            )
        }
    }

    fn put_phi_inputs_on_stack(&mut self, ctxt: &mut parser::Context) {
        for phi_var in self.bbs.get(&self.current_bb_id).unwrap().phi_inputs.iter() {
            ctxt.push_var(phi_var.clone());
        }
    }

    fn replace_phi_inputs_on_stack(&mut self, ctxt: &mut parser::Context) {
        for _ in self.bbs.get(&self.current_bb_id).unwrap().phi_inputs.iter() {
            ctxt.pop_var();
        }
        for phi in self.bbs.get(&self.current_bb_id).unwrap().phi_inputs.iter() {
            ctxt.push_var(phi.clone());
        }
    }

    fn continue_bb(&mut self, id: module::BasicBlockID) {
        if self.current_bb_id != u32::MAX {
            // store current instrs
            std::mem::swap(
                &mut self.current_instrs,
                &mut self.bbs.get_mut(&self.current_bb_id).unwrap().instrs,
            );
        }

        self.current_bb_id = id;
        // load new current instrs
        std::mem::swap(
            &mut self.current_instrs,
            &mut self.bbs.get_mut(&self.current_bb_id).unwrap().instrs,
        );
    }

    fn current_bb_get_else_marker_out_vars(&self) -> Option<SmallVec<[VariableID; 0]>> {
        self.bbs
            .get(&self.current_bb_id)
            .unwrap()
            .else_marker_out_vars
            .clone()
    }

    fn current_bb_id_get(&self) -> BasicBlockID {
        self.current_bb_id
    }

    fn current_bb_input_var_ids_get(&self) -> SmallVec<[VariableID; 1]> {
        self.bbs
            .get(&self.current_bb_id)
            .unwrap()
            .phi_inputs
            .iter()
            .map(|var| var.id)
            .collect()
    }

    fn current_bb_instrs(&mut self) -> &mut dyn module::InstructionConsumer {
        &mut self.current_instrs
    }

    fn terminate_jmp(
        &mut self,
        target: module::BasicBlockID,
        output_vars: SmallVec<[VariableID; 0]>,
    ) {
        self.add_phi_input_vals(target, output_vars.as_slice());
        let target_ctxt = self.bbs.get_mut(&target).unwrap();
        self.current_instrs
            .translator
            .builder
            .build_unconditional_branch(target_ctxt.bb);
    }

    fn terminate_else(&mut self, output_vars: SmallVec<[VariableID; 0]>) {
        self.bbs
            .get_mut(&self.current_bb_id)
            .unwrap()
            .else_marker_out_vars = Some(output_vars);
    }

    fn terminate_return(&mut self, return_vars: SmallVec<[VariableID; 1]>) {
        debug_assert!(
            !return_vars
                .iter()
                .any(|v| self.vars.borrow().len() <= *v || self.vars.borrow()[*v].is_null()),
            "Return variable does not have a value assigned."
        );
        match return_vars.len() {
            0 => self.current_instrs.translator.builder.build_ret_void(),
            1 => self
                .current_instrs
                .translator
                .builder
                .build_ret(self.vars.borrow()[return_vars[0]]),
            _ => {
                let mut return_values = return_vars
                    .iter()
                    .map(|var| self.vars.borrow()[*var])
                    .collect::<Vec<_>>();
                self.current_instrs
                    .translator
                    .builder
                    .build_aggregate_ret(&mut return_values);
            }
        };
    }
    fn terminate_unreachable(&mut self) {
        self.current_instrs.translator.builder.build_unreachable()
    }
    fn terminate_call_indirect(
        &mut self,
        type_idx: u32,
        selector_var: VariableID,
        table_idx: u32,
        return_bb: module::BasicBlockID,
        call_params: SmallVec<[VariableID; 2]>,
        return_vars: SmallVec<[VariableID; 1]>,
    ) {
        let selector_var = self.vars.borrow()[selector_var];
        let resolved_func_ptr = {
            let func_type = Module::create_func_type(
                self.current_instrs.translator.builder.ptr(),
                &mut [
                    self.current_instrs.translator.builder.ptr(),
                    self.current_instrs.translator.builder.i32(),
                    self.current_instrs.translator.builder.i32(),
                    self.current_instrs.translator.builder.i32(),
                ],
            );
            let indirect_call_fn = self
                .module
                .find_func("__wasmine_runtime.indirect_call", func_type)
                .unwrap_or_else(|| {
                    self.module.add_function(
                        "__wasmine_runtime.indirect_call",
                        func_type,
                        llvm_sys::LLVMLinkage::LLVMExternalLinkage,
                        llvm_sys::LLVMCallConv::LLVMCCallConv,
                    )
                });

            self.current_instrs.translator.builder.build_call(
                &indirect_call_fn,
                &mut [
                    Translator::get_rt_ref(&self.llvm_functions.borrow()[self.func_idx as usize]),
                    self.current_instrs.translator.builder.const_i32(table_idx),
                    self.current_instrs.translator.builder.const_i32(type_idx),
                    selector_var,
                ],
                "indirect_call_res",
            )
        };

        let indirect_fn_type = self.wasm_module.function_types[type_idx as usize];
        let mut param_types = vec![
            // runtime ptr
            self.current_instrs.translator.builder.ptr(),
        ];
        for valtype in indirect_fn_type.params_iter() {
            param_types.push(self.current_instrs.translator.builder.valtype2llvm(valtype));
        }

        let return_type = match indirect_fn_type.num_results() {
            0 => self.current_instrs.translator.builder.void(),
            1 => self
                .current_instrs
                .translator
                .builder
                .valtype2llvm(indirect_fn_type.results_iter().next().unwrap()),
            _ => self.current_instrs.translator.builder.r#struct(
                indirect_fn_type
                    .results_iter()
                    .map(|valtype| self.current_instrs.translator.builder.valtype2llvm(valtype))
                    .collect::<Vec<_>>()
                    .as_mut_slice(),
            ),
        };
        let indirect_llvm_fn_type = Module::create_func_type(return_type, &mut param_types);
        let func = Function::new(resolved_func_ptr, indirect_llvm_fn_type).unwrap();

        let mut parameters =
            vec![self.llvm_functions.borrow()[self.func_idx as usize].get_param(0)];
        for var_id in call_params {
            let llvm_val = self.vars.borrow()[var_id];
            parameters.push(llvm_val);
        }
        let indirect_func_call_res = self.current_instrs.translator.builder.build_call(
            &func,
            parameters.as_mut_slice(),
            if return_vars.is_empty() {
                ""
            } else {
                "call_indirect"
            },
        );
        match return_vars.len() {
            0 => (),
            1 => {
                if self.vars.borrow().len() <= return_vars[0] {
                    self.vars
                        .borrow_mut()
                        .resize(return_vars[0] + 1, std::ptr::null_mut());
                }
                self.vars.borrow_mut()[return_vars[0]] = indirect_func_call_res
            }
            _ => {
                let max_ret_var_idx = return_vars.iter().max().unwrap();
                if self.vars.borrow().len() <= *max_ret_var_idx {
                    self.vars
                        .borrow_mut()
                        .resize(max_ret_var_idx + 1, std::ptr::null_mut());
                }
                for (i, var) in return_vars.iter().enumerate() {
                    self.vars.borrow_mut()[*var] = unsafe {
                        LLVMBuildExtractValue(
                            self.current_instrs.translator.builder.get(),
                            indirect_func_call_res,
                            i as u32,
                            c_str("extract_res").as_ptr(),
                        )
                    };
                }
            }
        };
        self.current_instrs
            .translator
            .builder
            .build_unconditional_branch(self.bbs.get(&return_bb).unwrap().bb);
    }

    fn terminate_call(
        &mut self,
        func_idx: u32,
        return_bb: module::BasicBlockID,
        call_params: SmallVec<[VariableID; 2]>,
        return_vars: SmallVec<[VariableID; 1]>,
    ) {
        let function = &self.llvm_functions.borrow()[func_idx as usize];
        let mut parameters =
            vec![self.llvm_functions.borrow()[self.func_idx as usize].get_param(0)];
        for var_id in call_params {
            let llvm_val = self.vars.borrow()[var_id];
            parameters.push(llvm_val);
        }
        let call_res_val = self.current_instrs.translator.builder.build_call(
            function,
            parameters.as_mut_slice(),
            if return_vars.is_empty() {
                "" /* LLVM doesn't allow assigning names to void values */
            } else {
                "call_res"
            },
        );
        match return_vars.len() {
            0 => (),
            1 => {
                if self.vars.borrow().len() <= return_vars[0] {
                    self.vars
                        .borrow_mut()
                        .resize(return_vars[0] + 1, std::ptr::null_mut());
                }
                self.vars.borrow_mut()[return_vars[0]] = call_res_val;
            }
            _ => {
                let max_ret_var_idx = return_vars.iter().max().unwrap();
                if self.vars.borrow().len() <= *max_ret_var_idx {
                    self.vars
                        .borrow_mut()
                        .resize(max_ret_var_idx + 1, std::ptr::null_mut());
                }
                for (i, var) in return_vars.iter().enumerate() {
                    self.vars.borrow_mut()[*var] = unsafe {
                        LLVMBuildExtractValue(
                            self.current_instrs.translator.builder.get(),
                            call_res_val,
                            i as u32,
                            c_str("extract_res").as_ptr(),
                        )
                    };
                }
            }
        };
        self.current_instrs
            .translator
            .builder
            .build_unconditional_branch(self.bbs.get(&return_bb).unwrap().bb);
    }

    fn terminate_jmp_cond(
        &mut self,
        cond_var: VariableID,
        target_if_true: module::BasicBlockID,
        target_if_false: module::BasicBlockID,
        output_vars: SmallVec<[VariableID; 0]>,
    ) {
        self.add_phi_input_vals(target_if_true, output_vars.as_slice());
        self.add_phi_input_vals(target_if_false, output_vars.as_slice());
        let jmp_bool = self.current_instrs.translator.builder.build_icmp(
            llvm_sys::LLVMIntPredicate::LLVMIntNE,
            self.vars.borrow()[cond_var],
            self.current_instrs
                .translator
                .builder
                .const_zero(ValType::i32()),
            "ToBool",
        );
        self.current_instrs
            .translator
            .builder
            .build_conditional_branch(
                jmp_bool,
                self.bbs.get(&target_if_true).unwrap().bb,
                self.bbs.get(&target_if_false).unwrap().bb,
            )
    }

    fn terminate_jmp_table(
        &mut self,
        selector_var: VariableID,
        targets: SmallVec<[module::BasicBlockID; 5]>,
        targets_output_vars: SmallVec<[SmallVec<[VariableID; 0]>; 8]>,
        default_target: module::BasicBlockID,
        default_output_vars: SmallVec<[VariableID; 0]>,
    ) {
        let mut target_combinations = targets
            .iter()
            .chain([&default_target])
            .zip(targets_output_vars.iter().chain([&default_output_vars]))
            .collect::<SmallVec<[_; 3]>>();
        target_combinations.sort();
        target_combinations.dedup();
        for (target, target_out_vars) in target_combinations {
            self.add_phi_input_vals(*target, target_out_vars.as_slice());
        }

        let selector_val: *mut llvm_sys::LLVMValue = self.vars.borrow()[selector_var];
        let default_target_bb = self.bbs.get(&default_target).unwrap().bb;
        let switch = self.current_instrs.translator.builder.build_switch(
            selector_val,
            default_target_bb,
            targets.len() as u32,
        );
        for (i, target) in targets.iter().enumerate() {
            // exclude the default target, LLVM doesn't like targets that appear multiple times...
            if *target != default_target {
                unsafe {
                    llvm_sys::core::LLVMAddCase(
                        switch,
                        self.current_instrs.translator.builder.const_i32(i as u32),
                        self.bbs.get(&target).unwrap().bb,
                    )
                }
            }
        }
    }

    fn eliminate_current_bb(&mut self) {
        self.context
            .delete_basic_block(self.bbs.remove(&self.current_bb_id).unwrap().bb);
        self.current_bb_id = u32::MAX;
    }
}
