use crate::{
    globals::GlobalStorage, memory::MemoryInstance, segmented_list::SegmentedList,
    tables::TableInstance, Engine,
};
use runtime_interface::ExecutionContext;
use std::sync::Mutex;
use uuid::Uuid;

/// A cluster is a resource tracker, owning all allocated resources of its members. These resources include:
///  - Memories
///  - Tables
///  - Globals
///  - Functions (Closures)
///  - Engines
///  - Execution Contexts (which itself are just collections of pointers to this clusters resources)
///
/// Instance Handles are mere references to the resources and are therefore non-owning.
pub struct Cluster {
    pub(crate) uuid: Uuid,
    memories: Mutex<SegmentedList<MemoryInstance>>,
    tables: Mutex<SegmentedList<TableInstance>>,
    globals: Mutex<SegmentedList<GlobalStorage>>,
    execution_contexts: Mutex<SegmentedList<ExecutionContext>>,
    engines: Mutex<SegmentedList<Engine>>,
}

impl Cluster {
    pub fn new() -> Self {
        Self::default()
    }

    // unsafe: extracts mut-ref on cluster.globals without owning cluster.globals
    pub(crate) fn alloc_global_storage(&self, storage: GlobalStorage) -> &mut GlobalStorage {
        let mut globals_lock = self.globals.lock().unwrap();
        globals_lock.push(storage);
        &mut globals_lock.get_last_segments_ref()[0]
    }

    // unsafe: extracts mut-ref on cluster.tables without owning cluster.tables
    pub(crate) fn alloc_tables(&self, tables: Vec<TableInstance>) -> &mut [TableInstance] {
        let mut tables_lock = self.tables.lock().unwrap();
        tables_lock.extend(tables);
        tables_lock.get_last_segments_ref()
    }

    // unsafe: extracts mut-ref on cluster.memories without owning cluster.memories
    pub(crate) fn alloc_memories(&self, memories: Vec<MemoryInstance>) -> &mut [MemoryInstance] {
        let mut memories_lock = self.memories.lock().unwrap();
        memories_lock.extend(memories);
        memories_lock.get_last_segments_ref()
    }

    pub(crate) fn alloc_execution_context(
        &self,
        execution_context: ExecutionContext,
    ) -> &mut ExecutionContext {
        let mut execution_contexts_lock = self.execution_contexts.lock().unwrap();
        execution_contexts_lock.push(execution_context);
        &mut execution_contexts_lock.get_last_segments_ref()[0]
    }

    pub(crate) fn alloc_engine(&self, engine: Engine) -> &mut Engine {
        let mut engines_lock = self.engines.lock().unwrap();
        engines_lock.push(engine);
        &mut engines_lock.get_last_segments_ref()[0]
    }
}

impl PartialEq for Cluster {
    fn eq(&self, other: &Self) -> bool {
        self.uuid == other.uuid
    }
}

impl Default for Cluster {
    fn default() -> Self {
        Self {
            uuid: Uuid::new_v4(),
            memories: Mutex::new(SegmentedList::new()),
            tables: Mutex::new(SegmentedList::new()),
            globals: Mutex::new(SegmentedList::new()),
            execution_contexts: Mutex::new(SegmentedList::new()),
            engines: Mutex::new(SegmentedList::new()),
        }
    }
}
