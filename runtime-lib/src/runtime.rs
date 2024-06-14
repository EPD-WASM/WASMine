use runtime_interface::GlobalStorage;

use crate::{
    context::RTContext, error::RuntimeError, execution_context::ExecutionContext, globals,
    tables::TableInstance,
};

pub struct Runtime {
    pub config: RTContext,
    pub tables: Vec<TableInstance>,
    pub globals: GlobalStorage,
}
impl Runtime {
    /// This is the entrypoint for the wasm runtime. This entrypoint should only be called from Rust code.
    pub fn init(config: RTContext) -> *mut Self {
        let runtime = Box::new(Self {
            tables: Vec::new(),
            globals: globals::new(&config.module.globals),
            config,
        });
        Box::into_raw(runtime)
    }

    pub fn create_execution_context(&mut self) -> Result<ExecutionContext, RuntimeError> {
        ExecutionContext::init(
            unsafe { libc::gettid() } as u32,
            self,
            self.config.memories.clone(),
        )
    }
}
