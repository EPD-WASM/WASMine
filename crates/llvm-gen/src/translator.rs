use crate::abstraction::context::Context;
use crate::abstraction::function::Function;
use crate::abstraction::module::Module;
use crate::util::{build_llvm_function_name, c_str};
use crate::{abstraction::builder::Builder, error::TranslationError};
use llvm_sys::core::LLVMBuildExtractValue;
use llvm_sys::prelude::{LLVMBasicBlockRef, LLVMTypeRef, LLVMValueRef};
use llvm_sys::{LLVMCallConv, LLVMLinkage};
use module::objects::function::{
    Function as WasmFunction, FunctionIR as WasmFunctionIR, FunctionImport as WasmFunctionImport,
    FunctionUnparsed,
};
use module::{basic_block::BasicBlock, InstructionDecoder, ModuleMetadata as WasmModuleMeta};
use std::cell::RefCell;
use std::collections::HashSet;
use std::ptr::null_mut;
use std::rc::Rc;
use wasm_types::*;

pub struct Translator<'wasm> {
    pub(crate) builder: Builder,
    pub(crate) context: Rc<Context>,
    pub(crate) module: Rc<Module>,
    pub(crate) wasm_module_meta: &'wasm WasmModuleMeta,
    pub(crate) llvm_functions: Rc<RefCell<Vec<Function>>>,
}

impl<'wasm> Translator<'wasm> {
    pub fn translate_full_module(
        context: Rc<Context>,
        wasm_module_meta: &'wasm WasmModuleMeta,
    ) -> Result<Rc<Module>, TranslationError> {
        let (module, functions) =
            Translator::translate_module_meta(context.clone(), wasm_module_meta)?;
        Translator::translate_functions(
            context.clone(),
            wasm_module_meta,
            module.clone(),
            functions.clone(),
        )?;
        Ok(module)
    }

    pub(crate) fn translate_module_meta(
        context: Rc<Context>,
        wasm_module_meta: &'wasm WasmModuleMeta,
    ) -> Result<(Rc<Module>, Rc<RefCell<Vec<Function>>>), TranslationError> {
        let module = Rc::new(Module::new("main", &context));
        let builder = context.create_builder(module.clone());
        let mut instance = Self {
            context,
            builder,
            module,
            wasm_module_meta,
            llvm_functions: Rc::default(),
        };
        instance.translate_module_meta_internal()?;
        Ok((instance.module, instance.llvm_functions))
    }

    pub(crate) fn translate_functions(
        context: Rc<Context>,
        wasm_module_meta: &'wasm WasmModuleMeta,
        llvm_module: Rc<Module>,
        llvm_functions: Rc<RefCell<Vec<Function>>>,
    ) -> Result<(), TranslationError> {
        let builder = context.create_builder(llvm_module.clone());
        let instance = Self {
            context,
            builder,
            module: llvm_module,
            wasm_module_meta,
            llvm_functions,
        };

        for func_idx in 0..wasm_module_meta.functions.len() {
            let wasm_func_type = wasm_module_meta.functions[func_idx].type_idx;
            if let Some(ir) = wasm_module_meta.functions[func_idx].get_ir() {
                let llvm_func = instance.llvm_functions.borrow()[func_idx];
                instance.translate_internal_function(
                    &ir,
                    wasm_func_type,
                    &llvm_func,
                    func_idx as FuncIdx,
                )?;
            }
        }
        Ok(())
    }

    pub(crate) fn translate_single_function(
        context: Rc<Context>,
        wasm_module_meta: &'wasm WasmModuleMeta,
        llvm_module: Rc<Module>,
        llvm_functions: Rc<RefCell<Vec<Function>>>,
        ir: &WasmFunctionIR,
        func_idx: usize,
    ) -> Result<(), TranslationError> {
        let builder = context.create_builder(llvm_module.clone());
        let instance = Self {
            context,
            builder,
            module: llvm_module,
            wasm_module_meta,
            llvm_functions,
        };

        let wasm_func_type = wasm_module_meta.functions[func_idx].type_idx;
        let llvm_func = instance.llvm_functions.borrow()[func_idx];
        instance.translate_internal_function(
            &ir,
            wasm_func_type,
            &llvm_func,
            func_idx as FuncIdx,
        )?;
        Ok(())
    }

    fn translate_module_meta_internal(&mut self) -> Result<Rc<Module>, TranslationError> {
        for (global_idx, global) in self.wasm_module_meta.globals.iter().enumerate() {
            let name = format!("__wasmine_global__{global_idx}");
            let global_val_ty = match global.r#type {
                GlobalType::Const(vt) => vt,
                GlobalType::Mut(vt) => vt,
            };
            self.module
                .add_global(&name, self.builder.valtype2llvm(global_val_ty));
        }
        let required_rt_ptr_global_names = self
            .wasm_module_meta
            .imports
            .iter()
            .filter_map(|i| match i.desc {
                ImportDesc::Func(_) => Some(format!("{}.{}", i.module, i.name)),
                _ => None,
            })
            .collect::<HashSet<_>>();
        for rt_ctxt_name in required_rt_ptr_global_names {
            let name = format!("__import_ctxt__{rt_ctxt_name}__");
            self.module.add_global(&name, self.builder.ptr());
        }

        for (i, func) in self.wasm_module_meta.functions.iter().enumerate() {
            if let Some(WasmFunctionImport { import_idx }) = func.get_import().cloned() {
                let wasm_import = &self.wasm_module_meta.imports[import_idx as usize];
                let name = format!("{}.{}", wasm_import.module, wasm_import.name);
                self.llvm_functions
                    .borrow_mut()
                    .push(self.declare_imported_function(&name, i as FuncIdx, func.type_idx)?);
            } else if let Some(WasmFunctionIR { .. }) = func.get_ir() {
                self.llvm_functions
                    .borrow_mut()
                    .push(self.declare_internal_function(i as FuncIdx, func)?);
            } else if let Some(FunctionUnparsed { .. }) = func.get_unparsed_mem() {
                self.llvm_functions
                    .borrow_mut()
                    .push(self.declare_internal_function(i as FuncIdx, func)?);
            } else {
                panic!("unexpected function source: {func:?}");
            }
        }

        // create entrypoint wrappers for exported functions
        for (func_name, func_idx) in self.wasm_module_meta.exports.functions() {
            let wasm_function = &self.wasm_module_meta.functions[*func_idx as usize];
            let llvm_function = self.declare_export_wrapper(func_name)?;
            self.translate_external_wrapper_function(wasm_function, &llvm_function, *func_idx)?;
        }

        // add external wrapper for start function
        if let Some(start_func_idx) = self.wasm_module_meta.entry_point {
            let wasm_function = &self.wasm_module_meta.functions[start_func_idx as usize];
            let llvm_function = self.declare_export_wrapper(&build_llvm_function_name(
                start_func_idx,
                &self.wasm_module_meta,
                true,
            ))?;
            self.translate_external_wrapper_function(
                wasm_function,
                &llvm_function,
                start_func_idx,
            )?;
        }

        #[cfg(debug_assertions)]
        self.module.print_to_file();

        #[cfg(debug_assertions)]
        self.verify_module()?;

        Ok(self.module.clone())
    }

    pub(crate) fn llvm_external_func_type_from_wasm(
        &self,
    ) -> Result<LLVMTypeRef, TranslationError> {
        let mut param_types = vec![
            // runtime ptr
            self.builder.ptr(),
            // argument ptr
            self.builder.ptr(),
            // return parameter pointer
            self.builder.ptr(),
        ];
        Ok(Module::create_func_type(
            self.builder.void(),
            &mut param_types,
        ))
    }

    pub(crate) fn llvm_internal_func_type_from_wasm(
        &self,
        functype_idx: usize,
    ) -> Result<LLVMTypeRef, TranslationError> {
        let func_type = self.wasm_module_meta.function_types[functype_idx];
        let mut param_types = vec![
            // runtime ptr
            self.builder.ptr(),
        ];
        for valtype in func_type.params_iter() {
            param_types.push(self.builder.valtype2llvm(valtype));
        }

        let return_type = match func_type.num_results() {
            0 => self.builder.void(),
            1 => self
                .builder
                .valtype2llvm(func_type.results_iter().next().unwrap()),
            _ => self.builder.r#struct(
                func_type
                    .results_iter()
                    .map(|valtype| self.builder.valtype2llvm(valtype))
                    .collect::<Vec<_>>()
                    .as_mut_slice(),
            ),
        };
        Ok(Module::create_func_type(return_type, &mut param_types))
    }

    pub(crate) fn declare_internal_function(
        &self,
        function_idx: FuncIdx,
        function: &WasmFunction,
    ) -> Result<Function, TranslationError> {
        let fn_type = self.llvm_internal_func_type_from_wasm(function.type_idx as usize)?;
        let function_name = build_llvm_function_name(function_idx, &self.wasm_module_meta, false);
        let llvm_function = self.module.add_function(
            &function_name,
            fn_type,
            LLVMLinkage::LLVMExternalLinkage,
            LLVMCallConv::LLVMFastCallConv,
        );
        Ok(llvm_function)
    }

    pub(crate) fn declare_imported_function(
        &self,
        name: &str,
        func_idx: FuncIdx,
        type_idx: TypeIdx,
    ) -> Result<Function, TranslationError> {
        let fn_type = self.llvm_external_func_type_from_wasm()?;
        let internal_fn_type = self.llvm_internal_func_type_from_wasm(type_idx as usize)?;
        let internal_function_name = func_idx.to_string();
        if let Some(f) = self
            .module
            .find_func(&internal_function_name, internal_fn_type)
        {
            return Ok(f);
        }

        // declare imported function symbol
        let import_func_name = format!("__import__{name}__");
        let imported_function = self
            .module
            .find_func(&import_func_name, fn_type)
            .unwrap_or_else(|| {
                self.module.add_function(
                    &import_func_name,
                    fn_type,
                    LLVMLinkage::LLVMExternalLinkage,
                    LLVMCallConv::LLVMCCallConv,
                )
            });

        // create wrapper with internal signature for calls via different calling conventions
        let internal_fn_wasm_type = self.wasm_module_meta.function_types[type_idx as usize];
        let internal_function = self.module.add_function(
            &internal_function_name,
            internal_fn_type,
            LLVMLinkage::LLVMExternalLinkage,
            LLVMCallConv::LLVMFastCallConv,
        );
        let entry_bb = self
            .context
            .append_basic_block(internal_function.get(), "entry");
        self.builder.position_at_end(entry_bb);

        // don't forward current runtime ptr, but load their closured runtime ptr
        let import_ctxt_ptr = self
            .module
            .get_global(&format!("__import_ctxt__{name}__"))?;
        let mut params = vec![import_ctxt_ptr /* forward imported runtime ptr */];

        let param_arr_ptr: *mut llvm_sys::LLVMValue = self.builder.build_alloca(
            self.builder.array(
                self.builder.value_raw_ty(),
                internal_fn_wasm_type.num_params(),
            ),
            "param_arr",
        );
        for (i, _) in internal_fn_wasm_type.params_iter().enumerate() {
            let param_ptr = self.builder.build_gep(
                self.builder.value_raw_ty(),
                param_arr_ptr,
                &mut [self.builder.const_i32(i as u32)],
                &format!("function param {i} ptr calc"),
            );
            self.builder
                .build_store(internal_function.get_param(i + 1), param_ptr);
        }
        params.push(param_arr_ptr);
        // add return parameter pointer
        let return_param_ptr = self.builder.build_alloca(
            self.builder.array(
                self.builder.value_raw_ty(),
                internal_fn_wasm_type.num_results(),
            ),
            "return_param_arr",
        );
        params.push(return_param_ptr);
        self.builder
            .build_call(&imported_function, params.as_mut_slice(), "");

        match internal_fn_wasm_type.num_results() {
            0 => self.builder.build_ret_void(),
            1 => {
                let ret_val = self.builder.build_load(
                    self.builder
                        .valtype2llvm(internal_fn_wasm_type.results_iter().next().unwrap()),
                    return_param_ptr,
                    "ret_val_0_load",
                );
                self.builder.build_ret(ret_val);
            }
            _ => {
                let mut returns = Vec::new();
                for (i, wasm_ret_ty) in internal_fn_wasm_type.results_iter().enumerate() {
                    let ret_val_output_ptr = self.builder.build_gep(
                        self.builder.value_raw_ty(),
                        return_param_ptr,
                        &mut [self.builder.const_i32(i as u32)],
                        &format!("ret_val_{i}_out_ptr"),
                    );
                    let ret_val_elem = self.builder.build_load(
                        self.builder.valtype2llvm(wasm_ret_ty),
                        ret_val_output_ptr,
                        &format!("ret_val_{i}_load"),
                    );
                    returns.push(ret_val_elem);
                }
                self.builder.build_aggregate_ret(returns.as_mut_slice())
            }
        }

        #[cfg(debug_assertions)]
        Self::verify_function(&self.module, &internal_function, 0, &self.wasm_module_meta)?;

        // return original import declaration
        Ok(internal_function)
    }

    pub(crate) fn declare_export_wrapper(
        &self,
        public_func_name: &str,
    ) -> Result<Function, TranslationError> {
        let fn_type = self.llvm_external_func_type_from_wasm()?;
        let llvm_function = self.module.add_function(
            public_func_name,
            fn_type,
            LLVMLinkage::LLVMExternalLinkage,
            LLVMCallConv::LLVMCCallConv,
        );
        Ok(llvm_function)
    }

    fn translate_basic_block_map(
        &self,
        wasm_function: &WasmFunctionIR,
        llvm_function: &Function,
    ) -> Vec<LLVMBasicBlockRef> {
        let bbs = &wasm_function.bbs;
        let max_id = bbs.last().unwrap().id;
        let mut out = Vec::with_capacity((max_id + 1) as usize);
        for i in 0..=max_id {
            match bbs.binary_search_by_key(&i, |bb| bb.id) {
                Ok(id) => out.push(
                    self.context
                        .append_basic_block(llvm_function.get(), &format!("bb{id}")),
                ),
                Err(_) => out.push(null_mut()),
            }
        }
        out
    }

    fn translate_internal_function(
        &self,
        wasm_function: &WasmFunctionIR,
        wasm_ty_idx: TypeIdx,
        llvm_function: &Function,
        _function_idx: FuncIdx,
    ) -> Result<(), TranslationError> {
        let func_type = self
            .wasm_module_meta
            .function_types
            .get(wasm_ty_idx as usize)
            .unwrap();
        // allocate locals (function parameters + explicit locals) inside entry block
        let locals = self.allocate_locals(func_type, wasm_function, llvm_function)?;

        let mut variable_map = vec![null_mut() as LLVMValueRef; wasm_function.num_vars];
        let llvm_function_blocks = self.translate_basic_block_map(wasm_function, llvm_function);
        // this exists, because every parsed function has at least one terminator = one basic block
        let first_declared_bb =
            llvm_function_blocks[wasm_function.bbs.first().unwrap().id as usize];
        self.builder.build_unconditional_branch(first_declared_bb);
        for wasm_bb in wasm_function.bbs.iter() {
            let llvm_bb = llvm_function_blocks[wasm_bb.id as usize];
            self.translate_basic_block(
                wasm_bb,
                llvm_bb,
                &locals,
                &mut variable_map,
                &llvm_function_blocks,
                llvm_function,
            )?;
        }

        // fixup basic block inputs (= phi nodes) in second pass
        for wasm_bb in wasm_function.bbs.iter() {
            for phi in wasm_bb.inputs.iter() {
                let (mut basic_blocks, mut incoming_vars): (Vec<_>, Vec<_>) = phi
                    .inputs
                    .iter()
                    .map(|(bb, var)| (llvm_function_blocks[*bb as usize], variable_map[*var]))
                    .unzip();
                let phi_val = variable_map[phi.out];
                Builder::phi_add_incoming(phi_val, &mut incoming_vars, &mut basic_blocks);
            }
        }

        #[cfg(debug_assertions)]
        Self::verify_function(
            &self.module,
            llvm_function,
            _function_idx,
            self.wasm_module_meta,
        )?;
        Ok(())
    }

    fn translate_external_wrapper_function(
        &self,
        wasm_function: &WasmFunction,
        llvm_function: &Function,
        function_idx: FuncIdx,
    ) -> Result<(), TranslationError> {
        let func_type = self
            .wasm_module_meta
            .function_types
            .get(wasm_function.type_idx as usize)
            .unwrap();
        let wrapped_function = &self.llvm_functions.borrow()[function_idx as usize];
        let entry_bb = self
            .context
            .append_basic_block(llvm_function.get(), "entry");
        self.builder.position_at_end(entry_bb);

        let mut params = vec![llvm_function.get_param(0) /* runtime ptr is kept */];
        let param_arr_ptr = llvm_function.get_param(1);
        for (i, val_type) in func_type.params_iter().enumerate() {
            let param_llvm_type = self.builder.valtype2llvm(val_type);
            let param_ptr = self.builder.build_gep(
                self.builder.value_raw_ty(),
                param_arr_ptr,
                &mut [self.builder.const_i32(i as u32)],
                &format!("function param {i} ptr calc"),
            );
            let param_val = self.builder.build_load(
                param_llvm_type,
                param_ptr,
                &format!("function param {i} load"),
            );
            params.push(param_val);
        }
        let ret_val = self.builder.build_call(
            wrapped_function,
            params.as_mut_slice(),
            if func_type.num_results() == 0 {
                ""
            } else {
                "call_internal"
            },
        );

        match func_type.num_results() {
            0 => (),
            1 => {
                let ret_arr_ptr = llvm_function.get_param(2);
                self.builder.build_store(ret_val, ret_arr_ptr);
            }
            _ => {
                let ret_arr_ptr = llvm_function.get_param(2);
                for i in 0..func_type.num_results() {
                    let ret_val_output_ptr = self.builder.build_gep(
                        self.builder.value_raw_ty(),
                        ret_arr_ptr,
                        &mut [self.builder.const_i32(i as u32)],
                        &format!("ret_val_{i}_out_ptr"),
                    );
                    let ret_val_elem = unsafe {
                        LLVMBuildExtractValue(
                            self.builder.get(),
                            ret_val,
                            i as u32,
                            c_str("asdf").as_ptr(),
                        )
                    };
                    self.builder.build_store(ret_val_elem, ret_val_output_ptr);
                }
            }
        }
        self.builder.build_ret_void();

        #[cfg(debug_assertions)]
        Self::verify_function(
            &self.module,
            llvm_function,
            function_idx,
            &self.wasm_module_meta,
        )?;
        Ok(())
    }

    fn allocate_locals(
        &self,
        func_type: &FuncType,
        function: &WasmFunctionIR,
        llvm_function: &Function,
    ) -> Result<Vec<(LLVMValueRef, LLVMTypeRef)>, TranslationError> {
        let bb = self
            .context
            .append_basic_block(llvm_function.get(), "entry");
        self.builder.position_at_end(bb);

        let mut locals = Vec::new();
        for (i, param_wasm_type) in func_type.params_iter().enumerate() {
            let param_llvm_type = self.builder.valtype2llvm(param_wasm_type);
            let param_val = llvm_function.get_param(i + 1);
            let local = self
                .builder
                .build_alloca(param_llvm_type, &format!("local{i}"));
            self.builder.build_store(param_val, local);
            locals.push((local, param_llvm_type));
        }
        for i in func_type.num_params()..function.locals.len() {
            let local_ty = &function.locals[i];
            let local_llvm_type = self.builder.valtype2llvm(*local_ty);
            let local_llvm_storage = self
                .builder
                .build_alloca(local_llvm_type, &format!("local{i}"));
            self.builder
                .build_store(self.builder.const_zero(*local_ty), local_llvm_storage);
            locals.push((local_llvm_storage, local_llvm_type));
        }
        Ok(locals)
    }

    fn translate_basic_block(
        &self,
        wasm_bb: &BasicBlock,
        llvm_bb: LLVMBasicBlockRef,
        local_map: &[(LLVMValueRef, LLVMTypeRef)],
        variable_map: &mut [LLVMValueRef],
        function_bbs: &[LLVMBasicBlockRef],
        llvm_function: &Function,
    ) -> Result<(), TranslationError> {
        self.builder.position_at_end(llvm_bb);

        // collect inputs (if required because of multiple predecessors)
        for phi in wasm_bb.inputs.iter() {
            variable_map[phi.out] = self
                .builder
                .build_phi(self.builder.valtype2llvm(phi.r#type), "phi")
        }

        let mut decoder = InstructionDecoder::new(wasm_bb.instructions.clone());
        while let Ok(instruction) = decoder.read_instruction_type() {
            match instruction.clone() {
                InstructionType::Numeric(i) => self.translate_numeric(
                    i,
                    instruction,
                    &mut decoder,
                    variable_map,
                    llvm_function,
                )?,
                InstructionType::Reference(i) => {
                    self.translate_reference(i, instruction, &mut decoder, variable_map)?
                }
                InstructionType::Variable(i) => {
                    self.translate_variable(i, instruction, &mut decoder, variable_map, local_map)?
                }
                InstructionType::Memory(i) => self.translate_memory(
                    i,
                    instruction,
                    &mut decoder,
                    variable_map,
                    llvm_function,
                )?,
                InstructionType::Parametric(i) => {
                    self.translate_parametric(i, instruction, &mut decoder, variable_map)?
                }
                InstructionType::Control(_) => {
                    panic!("control instructions should never reach the llvm translator")
                }
                InstructionType::Meta(MetaInstructionType::PhiNode) => {
                    panic!("phi nodes should never reach llvm translation")
                }
                InstructionType::Table(i) => {
                    self.translate_table(i, instruction, &mut decoder, variable_map, llvm_function)?
                }
                _ => todo!("instruction {:?}", instruction),
            }
        }
        self.translate_terminator(
            &wasm_bb.terminator,
            variable_map,
            function_bbs,
            llvm_function,
        )?;
        Ok(())
    }
}

#[cfg(debug_assertions)]
mod debug_helper {
    use llvm_sys::{
        analysis::{LLVMVerifierFailureAction, LLVMVerifyFunction, LLVMVerifyModule},
        core::LLVMDisposeMessage,
    };

    use super::*;
    use std::{ffi::CStr, mem::MaybeUninit};

    impl Translator<'_> {
        pub(super) fn verify_module(&self) -> Result<(), TranslationError> {
            let mut msg = MaybeUninit::uninit();
            let res = unsafe {
                LLVMVerifyModule(
                    self.module.get(),
                    LLVMVerifierFailureAction::LLVMPrintMessageAction,
                    msg.as_mut_ptr(),
                )
            };
            if res != 0 {
                let msg = unsafe { msg.assume_init() };
                if !msg.is_null() {
                    let res = Err(TranslationError::Msg(
                        unsafe { CStr::from_ptr(msg) }.to_string_lossy().into(),
                    ));
                    unsafe { LLVMDisposeMessage(msg) };
                    return res;
                } else {
                    return Err(TranslationError::Msg("unknown error".into()));
                }
            }
            Ok(())
        }

        pub(crate) fn verify_function(
            module: &Module,
            function: &Function,
            function_idx: u32,
            wasm_module_meta: &WasmModuleMeta,
        ) -> Result<(), TranslationError> {
            if 1 == unsafe {
                LLVMVerifyFunction(
                    function.get(),
                    LLVMVerifierFailureAction::LLVMPrintMessageAction,
                )
            } {
                // print module early for debugging
                module.print_to_file();
                return Err(TranslationError::Msg(format!(
                    "function verification failed for function {}",
                    build_llvm_function_name(function_idx, wasm_module_meta, false)
                )));
            }
            Ok(())
        }
    }
}
