use crate::{
    abstraction::{
        context::Context, lljit::JITExecutionEngine, module::Module, pass_manager::PassManager,
    },
    aot::AOTFunctions,
    error::ExecutionError,
    LLVMAdditionalResources,
};
use module::{objects::value::ValueRaw, Module as WasmModule};
use runtime_interface::RawPointer;
use std::rc::Rc;
use wasm_types::GlobalIdx;

pub struct JITExecutor {
    execution_engine: JITExecutionEngine,
    #[allow(dead_code)] // hold on to the context to prevent it from being dropped
    context: Rc<Context>,
}

impl JITExecutor {
    pub fn new(module: Rc<WasmModule>) -> Result<Self, ExecutionError> {
        if let Some(obj_buf) = module.artifact_registry.read().unwrap().get("llvm-obj") {
            let mut instance = Self {
                execution_engine: JITExecutionEngine::init()?,
                context: Rc::new(Context::create()),
            };
            let obj_buf = obj_buf.read().unwrap();
            let obj_buf = obj_buf.downcast_ref::<AOTFunctions>().unwrap();
            instance.add_object_file(
                &module.source.get()[obj_buf.offset..obj_buf.offset + obj_buf.size],
            )?;
            return Ok(instance);
        }

        let (llvm_module, llvm_context) = {
            let artifacts_ref = module.artifact_registry.read().unwrap();
            let llvm_resources = artifacts_ref.get("llvm-module").ok_or_else(|| {
                ExecutionError::Msg("LLVM module not found in artifact registry. Translate meta using `llvm-gen` first.".to_string())
            })?;
            let llvm_resources = llvm_resources.read().unwrap();
            let llvm_resources = llvm_resources
                .downcast_ref::<LLVMAdditionalResources>()
                .unwrap();
            (
                llvm_resources.module.clone(),
                llvm_resources.context.clone(),
            )
        };
        let mut instance = Self {
            execution_engine: JITExecutionEngine::init()?,
            context: llvm_context,
        };
        instance.add_module(llvm_module)?;
        return Ok(instance);
    }

    pub fn get_symbol_addr(&self, name: &str) -> Result<RawPointer, ExecutionError> {
        self.execution_engine.get_symbol_addr(name)
    }

    pub fn set_symbol_addr(&mut self, name: &str, address: RawPointer) {
        self.execution_engine.register_symbol(name, address);
    }

    pub fn add_module(&mut self, llvm_module: Rc<Module>) -> Result<(), ExecutionError> {
        PassManager::optimize_module(&llvm_module)?;

        #[cfg(debug_assertions)]
        llvm_module.print_to_file();

        self.execution_engine.add_llvm_module(llvm_module)
    }

    pub(crate) fn get_module_as_object_buffer(
        &self,
        module_idx: usize,
    ) -> Result<&[u8], ExecutionError> {
        self.execution_engine
            .get_module_as_object_buffer(module_idx)
    }

    /// Add object file to the JIT compiler
    ///
    /// # Warning
    /// Does not take ownership of the object file buffer. Buffer must be kept alive until the JIT compiler is dropped.
    pub(crate) fn add_object_file(&mut self, obj_file: &[u8]) -> Result<(), ExecutionError> {
        self.execution_engine.add_object_file(obj_file)
    }

    pub fn get_global_value(&self, global_idx: GlobalIdx) -> Result<ValueRaw, ExecutionError> {
        self.execution_engine
            .get_global(&format!("__wasmine_global__{global_idx}"))
    }

    pub fn set_global_addr(&mut self, global_idx: GlobalIdx, addr: RawPointer) {
        self.execution_engine
            .register_symbol(&format!("__wasmine_global__{global_idx}"), addr)
    }
}
