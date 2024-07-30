use super::module::Module;
use crate::util::to_c_str;
use crate::ExecutionError;
use ir::structs::value::ValueRaw;
use llvm_sys::error::{LLVMCreateStringError, LLVMErrorRef};
use llvm_sys::execution_engine::{LLVMCreateGDBRegistrationListener, LLVMLinkInMCJIT};
use llvm_sys::orc2::ee::{
    LLVMOrcCreateRTDyldObjectLinkingLayerWithSectionMemoryManager,
    LLVMOrcRTDyldObjectLinkingLayerRegisterJITEventListener,
};
use llvm_sys::orc2::lljit::{
    LLVMOrcCreateLLJIT, LLVMOrcCreateLLJITBuilder, LLVMOrcDisposeLLJIT,
    LLVMOrcLLJITAddLLVMIRModuleWithRT, LLVMOrcLLJITBuilderSetJITTargetMachineBuilder,
    LLVMOrcLLJITBuilderSetObjectLinkingLayerCreator, LLVMOrcLLJITGetExecutionSession,
    LLVMOrcLLJITGetMainJITDylib, LLVMOrcLLJITLookup, LLVMOrcLLJITRef,
};
use llvm_sys::orc2::{
    LLVMJITEvaluatedSymbol, LLVMJITSymbolFlags, LLVMJITSymbolGenericFlags, LLVMOrcAbsoluteSymbols,
    LLVMOrcCLookupSet, LLVMOrcCSymbolMapPair, LLVMOrcCreateCustomCAPIDefinitionGenerator,
    LLVMOrcCreateNewThreadSafeContext, LLVMOrcCreateNewThreadSafeModule,
    LLVMOrcDefinitionGeneratorRef, LLVMOrcExecutionSessionIntern, LLVMOrcExecutionSessionRef,
    LLVMOrcExecutionSessionSetErrorReporter, LLVMOrcJITDylibAddGenerator,
    LLVMOrcJITDylibCreateResourceTracker, LLVMOrcJITDylibDefine, LLVMOrcJITDylibLookupFlags,
    LLVMOrcJITDylibRef, LLVMOrcJITTargetMachineBuilderCreateFromTargetMachine, LLVMOrcLookupKind,
    LLVMOrcLookupStateRef, LLVMOrcMaterializationUnitRef, LLVMOrcObjectLayerRef,
    LLVMOrcSymbolStringPoolEntryStr,
};
use llvm_sys::target::LLVM_InitializeNativeTarget;
use llvm_sys::target_machine::{
    LLVMCodeGenOptLevel, LLVMCodeModel, LLVMCreateTargetMachine, LLVMGetDefaultTargetTriple,
    LLVMGetFirstTarget, LLVMGetHostCPUFeatures, LLVMGetHostCPUName, LLVMGetTargetFromTriple,
    LLVMRelocMode, LLVMTargetHasJIT, LLVMTargetMachineRef, LLVMTargetRef,
};
use llvm_sys::transforms::pass_builder::{LLVMCreatePassBuilderOptions, LLVMRunPasses};
use once_cell::sync::Lazy;
use runtime_interface::RawFunctionPtr;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::CStr;
use std::mem::{ManuallyDrop, MaybeUninit};
use std::ptr::null_mut;
use std::rc::Rc;
use std::sync::RwLock;

static TARGET_LOCK: Lazy<RwLock<()>> = Lazy::new(|| RwLock::new(()));

struct DynamicSymbolResolverContext {
    execution_session: LLVMOrcExecutionSessionRef,
    external_syms: Rc<RefCell<HashMap<String, *const core::ffi::c_void>>>,
}

pub(crate) struct ExecutionEngine {
    jit: LLVMOrcLLJITRef,

    // imported symbols
    external_syms: Rc<RefCell<HashMap<String, *const core::ffi::c_void>>>,

    // keep modules alive until the engine is dropped
    _modules: Vec<Rc<Module>>,

    // keep symbol resolution context for cleanup on destruction
    _symbol_gen_ctxt: *mut DynamicSymbolResolverContext,
}

impl ExecutionEngine {
    #[allow(dead_code)]
    pub(crate) fn has_jit() -> Result<bool, ExecutionError> {
        let mut target: MaybeUninit<LLVMTargetRef> = MaybeUninit::uninit();
        let mut error = MaybeUninit::uninit();
        if 0 != unsafe {
            LLVMGetTargetFromTriple(
                LLVMGetDefaultTargetTriple(),
                target.as_mut_ptr(),
                error.as_mut_ptr(),
            )
        } {
            return Err(ExecutionError::from(unsafe { error.assume_init() }));
        }
        Ok(unsafe { LLVMTargetHasJIT(target.assume_init()) } == 0)
    }

    extern "C" fn obj_linking_layer_creator(
        _: *mut ::libc::c_void,
        execution_session: LLVMOrcExecutionSessionRef,
        _: *const ::libc::c_char,
    ) -> LLVMOrcObjectLayerRef {
        let object_layer = unsafe {
            LLVMOrcCreateRTDyldObjectLinkingLayerWithSectionMemoryManager(execution_session)
        };
        #[cfg(debug_assertions)]
        unsafe {
            LLVMOrcRTDyldObjectLinkingLayerRegisterJITEventListener(
                object_layer,
                LLVMCreateGDBRegistrationListener(),
            )
        };
        object_layer
    }

    extern "C" fn llvm_log_jit_error(_: *mut ::libc::c_void, e: LLVMErrorRef) {
        log::error!("JIT error: {:?}", ExecutionError::from(e))
    }

    // callback called for all unresolved symbols discovered during JIT compilation
    extern "C" fn dynamic_symbol_resolver(
        _generator_obj: LLVMOrcDefinitionGeneratorRef,
        ctxt: *mut ::libc::c_void,
        _lookup_state: *mut LLVMOrcLookupStateRef,
        _kind: LLVMOrcLookupKind,
        jd: LLVMOrcJITDylibRef,
        _jd_lookup_flags: LLVMOrcJITDylibLookupFlags,
        lookup_set: LLVMOrcCLookupSet,
        lookup_set_size: usize,
    ) -> LLVMErrorRef {
        let ctxt =
            unsafe { ManuallyDrop::new(Box::from_raw(ctxt as *mut DynamicSymbolResolverContext)) };
        let borrowed_syms = ctxt.external_syms.borrow();
        let mut symbols = OrcSymbolMap::new();
        for i in (0..lookup_set_size).rev() {
            let lookup = unsafe { lookup_set.add(i) };
            let name = unsafe { CStr::from_ptr(LLVMOrcSymbolStringPoolEntryStr((*lookup).Name)) };
            log::debug!("Custom symbol lookup: {:?}", name);
            let name_str = match name.to_str() {
                Ok(s) => s,
                Err(e) => {
                    return unsafe {
                        LLVMCreateStringError(
                            to_c_str(&format!("Error during external method resolution: {}", e))
                                .as_ptr(),
                        )
                    };
                }
            };
            let sym_addr = match borrowed_syms.get(name_str) {
                Some(addr) => addr,
                None => {
                    return unsafe {
                        LLVMCreateStringError(
                            to_c_str(&format!(
                                "Could not find imported symbol '{}' in {:?}",
                                name_str, ctxt.external_syms
                            ))
                            .as_ptr(),
                        )
                    };
                }
            };
            symbols.insert(name, *sym_addr, ctxt.execution_session);
        }
        unsafe { LLVMOrcJITDylibDefine(jd, symbols.into_absolute_symbols_materialization_unit()) }
    }

    extern "C" fn custom_symbol_generator_destructor(_: *mut ::libc::c_void) {
        // noop
    }

    fn create_target_machine() -> Result<LLVMTargetMachineRef, ExecutionError> {
        let cpu = unsafe { LLVMGetHostCPUName() };
        log::debug!("LLVM detected CPU: {:?}", unsafe { CStr::from_ptr(cpu) });

        let features = unsafe { LLVMGetHostCPUFeatures() };
        log::debug!("LLVM detected CPU features: {:?}", unsafe {
            CStr::from_ptr(features)
        });

        let target_triple = unsafe { LLVMGetDefaultTargetTriple() };
        log::debug!("LLVM default target triple: {:?}", unsafe {
            CStr::from_ptr(target_triple)
        });
        if 1 == unsafe { LLVM_InitializeNativeTarget() } {
            return Err(ExecutionError::LLVM(
                "Failed to initialize native target".into(),
            ));
        }
        let target = unsafe { LLVMGetFirstTarget() };
        let target_machine = unsafe {
            LLVMCreateTargetMachine(
                target,
                target_triple,
                cpu,
                features,
                LLVMCodeGenOptLevel::LLVMCodeGenLevelDefault,
                LLVMRelocMode::LLVMRelocDefault,
                LLVMCodeModel::LLVMCodeModelJITDefault,
            )
        };
        Ok(target_machine)
    }

    pub(crate) fn optimize_module(&mut self, module: &Module) -> Result<(), ExecutionError> {
        let options = unsafe { LLVMCreatePassBuilderOptions() };
        let err = unsafe {
            LLVMRunPasses(
                module.get(),
                to_c_str("simplifycfg").as_ptr(),
                Self::create_target_machine()?,
                options,
            )
        };
        if !err.is_null() {
            return Err(err.into());
        }
        Ok(())
    }

    pub(crate) fn add_llvm_module(&mut self, module: Rc<Module>) -> Result<(), ExecutionError> {
        // add llvm module to jit compiler
        let main_dylib = unsafe { LLVMOrcLLJITGetMainJITDylib(self.jit) };
        let ts_ctxt = unsafe { LLVMOrcCreateNewThreadSafeContext() };
        let thread_safe_llvm_module =
            unsafe { LLVMOrcCreateNewThreadSafeModule(module.get(), ts_ctxt) };
        let resource_tracker = unsafe { LLVMOrcJITDylibCreateResourceTracker(main_dylib) };
        let error = unsafe {
            LLVMOrcLLJITAddLLVMIRModuleWithRT(self.jit, resource_tracker, thread_safe_llvm_module)
        };
        if !error.is_null() {
            return Err(error.into());
        }
        self._modules.push(module);
        Ok(())
    }

    pub(crate) fn register_symbol(&mut self, name: &str, addr: *const core::ffi::c_void) {
        self.external_syms.borrow_mut().insert(name.into(), addr);
    }

    pub(crate) fn init() -> Result<Self, ExecutionError> {
        unsafe {
            // this is a noop-function forcing linkage of the JIT compiler
            // DON'T REMOVE OR YOU'LL ENCOUNTER THE ERROR "JIT has not been linked in."
            LLVMLinkInMCJIT();
        }

        // initialize llvm / target
        {
            {
                let _guard = TARGET_LOCK.write().unwrap_or_else(|e| e.into_inner());
                let code = unsafe { llvm_sys::target::LLVM_InitializeNativeTarget() };

                if code == 1 {
                    return Err(ExecutionError::LLVM(
                        "Unknown error in initializing native target".into(),
                    ));
                }
            }
            {
                let _guard = TARGET_LOCK.write().unwrap_or_else(|e| e.into_inner());
                let code = unsafe { llvm_sys::target::LLVM_InitializeNativeAsmPrinter() };

                if code == 1 {
                    return Err(ExecutionError::LLVM(
                        "Unknown error in initializing native asm printer".into(),
                    ));
                }
            }
        }

        let lljit_builder = unsafe { LLVMOrcCreateLLJITBuilder() };

        // set JIT target machine as host machine
        let target_machine = Self::create_target_machine()?;
        unsafe {
            let jit_target_machine_builder =
                LLVMOrcJITTargetMachineBuilderCreateFromTargetMachine(target_machine);
            LLVMOrcLLJITBuilderSetJITTargetMachineBuilder(lljit_builder, jit_target_machine_builder)
        };

        // add object linking layer creator function pointer
        unsafe {
            LLVMOrcLLJITBuilderSetObjectLinkingLayerCreator(
                lljit_builder,
                Self::obj_linking_layer_creator,
                /* ctxt pointer for creator func */ null_mut(),
            )
        };

        // create jit compiler
        let mut lljit: MaybeUninit<LLVMOrcLLJITRef> = MaybeUninit::uninit();
        let error = unsafe { LLVMOrcCreateLLJIT(lljit.as_mut_ptr(), lljit_builder) };
        if !error.is_null() {
            return Err(error.into());
        }
        let lljit = unsafe { lljit.assume_init() };

        // set error reporter
        unsafe {
            LLVMOrcExecutionSessionSetErrorReporter(
                LLVMOrcLLJITGetExecutionSession(lljit),
                Self::llvm_log_jit_error,
                null_mut(),
            );
        }

        // create dynamic (custom) symbol resolver
        let external_syms = Rc::new(RefCell::new(HashMap::new()));
        let symbol_gen_ctxt = Box::into_raw(Box::new(DynamicSymbolResolverContext {
            execution_session: unsafe { LLVMOrcLLJITGetExecutionSession(lljit) },
            external_syms: external_syms.clone(),
        })) as *mut ::libc::c_void;
        let symbol_gen = unsafe {
            LLVMOrcCreateCustomCAPIDefinitionGenerator(
                Self::dynamic_symbol_resolver,
                symbol_gen_ctxt,
                Self::custom_symbol_generator_destructor,
            )
        };
        unsafe {
            LLVMOrcJITDylibAddGenerator(LLVMOrcLLJITGetMainJITDylib(lljit), symbol_gen);
        }

        Ok(Self {
            jit: lljit,
            external_syms,
            _modules: Vec::new(),
            _symbol_gen_ctxt: symbol_gen_ctxt as *mut DynamicSymbolResolverContext,
        })
    }

    pub(crate) fn get(&self) -> LLVMOrcLLJITRef {
        self.jit
    }

    pub fn find_func_address_by_name(
        &self,
        fn_name: &str,
    ) -> Result<RawFunctionPtr, ExecutionError> {
        let mut address = MaybeUninit::uninit();
        let error = unsafe {
            LLVMOrcLLJITLookup(self.get(), address.as_mut_ptr(), to_c_str(fn_name).as_ptr())
        };
        if !error.is_null() {
            return Err(ExecutionError::from(error));
        }
        let address = unsafe { address.assume_init() };
        RawFunctionPtr::new(address as *mut _).ok_or(ExecutionError::FunctionNotFound)
    }

    pub fn get_global_value(&self, global_name: &str) -> Result<ValueRaw, ExecutionError> {
        let addr = self.find_func_address_by_name(global_name)?;
        Ok(unsafe { std::ptr::read(addr.cast().as_ptr()) })
    }

    #[allow(dead_code)]
    #[cfg(debug_assertions)]
    pub(crate) fn write_to_file(&self) -> Result<(), ExecutionError> {
        use llvm_sys::target_machine::{LLVMCodeGenFileType, LLVMTargetMachineEmitToFile};

        let file_name = to_c_str("debug.o");
        let mut error_msg = MaybeUninit::uninit();
        if 0 != unsafe {
            LLVMTargetMachineEmitToFile(
                Self::create_target_machine()?,
                self._modules[0].get(),
                std::mem::transmute::<*const i8, *mut i8>(file_name.as_ptr()),
                LLVMCodeGenFileType::LLVMObjectFile,
                error_msg.as_mut_ptr(),
            )
        } {
            let error_msg = unsafe { error_msg.assume_init() };
            return Err(ExecutionError::from(error_msg));
        }
        Ok(())
    }
}

impl Drop for ExecutionEngine {
    fn drop(&mut self) {
        unsafe {
            drop(Box::from_raw(self._symbol_gen_ctxt));
            LLVMOrcDisposeLLJIT(self.jit);
        }
    }
}

struct OrcSymbolMap {
    inner: Vec<LLVMOrcCSymbolMapPair>,
}

impl OrcSymbolMap {
    fn new() -> Self {
        Self { inner: Vec::new() }
    }

    fn insert(
        &mut self,
        name: &CStr,
        addr: *const core::ffi::c_void,
        es: LLVMOrcExecutionSessionRef,
    ) {
        self.inner.push(LLVMOrcCSymbolMapPair {
            Name: unsafe { LLVMOrcExecutionSessionIntern(es, name.as_ptr()) },
            Sym: LLVMJITEvaluatedSymbol {
                Address: addr as u64,
                Flags: LLVMJITSymbolFlags {
                    GenericFlags: LLVMJITSymbolGenericFlags::LLVMJITSymbolGenericFlagsExported
                        as u8,
                    TargetFlags: 0,
                },
            },
        });
    }

    fn into_absolute_symbols_materialization_unit(mut self) -> LLVMOrcMaterializationUnitRef {
        unsafe { LLVMOrcAbsoluteSymbols(self.inner.as_mut_ptr(), self.inner.len()) }
    }
}
