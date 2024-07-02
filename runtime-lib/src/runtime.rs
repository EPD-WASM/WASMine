use crate::{
    error::RuntimeError, execution_context::ExecutionContextWrapper, globals::GlobalStorage,
    tables::TableInstance,
};
use ir::structs::module::Module;
use std::{pin::Pin, rc::Rc};

pub struct Runtime {
    pub(crate) tables: Vec<TableInstance>,
    pub(crate) globals: GlobalStorage,
    pub(crate) module: Rc<Module>,
}

impl Runtime {
    pub fn new(module: Rc<Module>) -> Self {
        Self {
            tables: Vec::new(),
            globals: GlobalStorage::new(&module.globals),
            module,
        }
    }

    pub fn create_execution_context(
        self: &Pin<Box<Self>>,
        imported_memories: &[runtime_interface::MemoryInstance],
    ) -> Result<runtime_interface::ExecutionContext, RuntimeError> {
        let unsafe_rt_ptr = Pin::as_ref(self).get_ref() as *const Runtime;
        ExecutionContextWrapper::init(
            unsafe { libc::gettid() } as u32,
            unsafe_rt_ptr as *mut Runtime,
            &self.module,
            imported_memories,
        )
    }

    pub fn globals_ref(&self) -> &runtime_interface::GlobalStorage {
        &self.globals.inner
    }
}
