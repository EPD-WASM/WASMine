use crate::{
    abstraction::function::Function,
    util::{build_llvm_function_name, to_c_str},
    TranslationError, Translator,
};
use ir::basic_block::BasicBlockGlue;
use llvm_sys::{
    core::LLVMBuildExtractValue,
    prelude::{LLVMBasicBlockRef, LLVMValueRef},
};
use wasm_types::{NumType, ValType};

impl Translator {
    pub(crate) fn translate_terminator(
        &self,
        terminator: &BasicBlockGlue,
        variable_map: &mut [LLVMValueRef],
        function_bbs: &[LLVMBasicBlockRef],
        llvm_function: &Function,
    ) -> Result<(), TranslationError> {
        match terminator {
            BasicBlockGlue::Jmp { target, .. } => {
                self.builder
                    .build_unconditional_branch(function_bbs[*target as usize]);
            }
            BasicBlockGlue::JmpCond {
                cond_var,
                target_if_true,
                target_if_false,
                ..
            } => {
                let jmp_bool = self.builder.build_icmp(
                    llvm_sys::LLVMIntPredicate::LLVMIntNE,
                    variable_map[*cond_var as usize],
                    self.builder.const_zero(ValType::Number(NumType::I32)),
                    "ToBool",
                );
                self.builder.build_conditional_branch(
                    jmp_bool,
                    function_bbs[*target_if_true as usize],
                    function_bbs[*target_if_false as usize],
                )
            }
            BasicBlockGlue::Return { return_vars } => {
                debug_assert!(
                    !return_vars
                        .iter()
                        .any(|v| { variable_map[*v as usize].is_null() }),
                    "Return variable does not have a value assigned."
                );
                match return_vars.len() {
                    0 => self.builder.build_ret_void(),
                    1 => self
                        .builder
                        .build_ret(variable_map[return_vars[0] as usize]),
                    _ => {
                        let mut return_values = return_vars
                            .iter()
                            .map(|var| variable_map[*var as usize])
                            .collect::<Vec<_>>();
                        self.builder.build_aggregate_ret(&mut return_values);
                    }
                };
            }
            BasicBlockGlue::Call {
                func_idx,
                return_bb,
                call_params,
                return_vars,
            } => {
                // TODO: remove this
                #[cfg(debug_assertions)]
                let _called_fun_name =
                    build_llvm_function_name(*func_idx, &self.wasm_module, false);

                let function = &self.llvm_functions[*func_idx as usize];
                let mut parameters = vec![llvm_function.get_param(0)];
                for var_id in call_params.iter() {
                    let llvm_val = variable_map[*var_id as usize];
                    parameters.push(llvm_val);
                }
                let call_res_val = self.builder.build_call(
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
                        variable_map[return_vars[0] as usize] = call_res_val;
                    }
                    _ => {
                        for (i, var) in return_vars.iter().enumerate() {
                            variable_map[*var as usize] = unsafe {
                                LLVMBuildExtractValue(
                                    self.builder.get(),
                                    call_res_val,
                                    i as u32,
                                    to_c_str("extract_res").as_ptr(),
                                )
                            };
                        }
                    }
                };
                self.builder
                    .build_unconditional_branch(function_bbs[*return_bb as usize]);
            }
            BasicBlockGlue::Unreachable => self.builder.build_unreachable(),
            BasicBlockGlue::JmpTable {
                selector_var: cond_var,
                targets,
                default_target,
                ..
            } => {
                let selector_val = variable_map[*cond_var as usize];
                let default_target_bb = function_bbs[*default_target as usize];
                let switch = self.builder.build_switch(
                    selector_val,
                    default_target_bb,
                    targets.len() as u32,
                );
                for (i, target) in targets.iter().enumerate() {
                    // exclude the default target, LLVM doesn't like targets that appear multiple times...
                    if target != default_target {
                        unsafe {
                            llvm_sys::core::LLVMAddCase(
                                switch,
                                self.builder.const_i32(i as u32),
                                function_bbs[*target as usize],
                            )
                        }
                    }
                }
            }
            BasicBlockGlue::CallIndirect {
                type_idx,
                selector_var,
                table_idx,
                return_bb,
                call_params,
                return_vars,
            } => {
                let selector_var = variable_map[*selector_var as usize];
                let resolved_func_ptr = self.indirect_call(
                    Self::get_rt_ref(llvm_function),
                    *table_idx,
                    *type_idx,
                    selector_var,
                );
                let indirect_func_llvm_type =
                    self.llvm_internal_func_type_from_wasm(*type_idx as usize)?;
                let func = Function::new(resolved_func_ptr, indirect_func_llvm_type).unwrap();

                let mut parameters = vec![llvm_function.get_param(0)];
                for var_id in call_params {
                    let llvm_val = variable_map[*var_id as usize];
                    parameters.push(llvm_val);
                }
                let indirect_func_call_res = self.builder.build_call(
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
                    1 => variable_map[return_vars[0] as usize] = indirect_func_call_res,
                    _ => {
                        for (i, var) in return_vars.iter().enumerate() {
                            variable_map[*var as usize] = unsafe {
                                LLVMBuildExtractValue(
                                    self.builder.get(),
                                    indirect_func_call_res,
                                    i as u32,
                                    to_c_str("extract_res").as_ptr(),
                                )
                            };
                        }
                    }
                };
                self.builder
                    .build_unconditional_branch(function_bbs[*return_bb as usize]);
            }
            BasicBlockGlue::ElseMarker { .. } => {
                return Err(TranslationError::Unimplemented(
                    "else marker should never reach the llvm compiler".into(),
                ))
            }
        };
        Ok(())
    }
}
