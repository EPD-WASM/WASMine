use wasm_types::GlobalStorage;

use crate::{
    context::RTContext, error::RuntimeError, execution_context::ExecutionContext, globals,
    tables::TableInstance,
};

pub(crate) struct Runtime {
    pub(crate) config: RTContext,
    pub(crate) tables: Vec<TableInstance>,
    pub(crate) globals: GlobalStorage,
}
impl Runtime {
    /// This is the entrypoint for the wasm runtime. This entrypoint should only be called from Rust code.
    pub(crate) fn init(config: RTContext) -> *mut Self {
        let runtime = Box::new(Self {
            tables: Vec::new(),
            globals: globals::new(&config.module.globals),
            config,
        });
        Box::into_raw(runtime)
    }

    pub(crate) fn create_execution_context(&mut self) -> Result<ExecutionContext, RuntimeError> {
        ExecutionContext::init(
            unsafe { libc::gettid() } as u32,
            self,
            self.config.memories.clone(),
        )
    }
}
