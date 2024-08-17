use crate::{
    helper::segmented_list::SegmentedList,
    objects::{
        functions::Function, globals::GlobalsObject, memory::MemoryObject, tables::TableObject,
    },
    wasi::WasiContext,
    Engine,
};
use runtime_interface::ExecutionContext;
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Default)]
pub struct ClusterConfig {/* TODO */}

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
    pub(crate) config: ClusterConfig,
    memories: Mutex<SegmentedList<MemoryObject>>,
    tables: Mutex<SegmentedList<TableObject>>,
    globals: Mutex<SegmentedList<GlobalsObject>>,
    execution_contexts: Mutex<SegmentedList<ExecutionContext>>,
    engines: Mutex<SegmentedList<Engine>>,
    functions: Mutex<SegmentedList<Function>>,
    wasi_ctxt: Mutex<SegmentedList<WasiContext>>,
}

impl Cluster {
    pub fn new(config: ClusterConfig) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    // unsafe: extracts mut-ref on cluster.globals without owning cluster.globals
    pub(crate) fn alloc_global_storage(&self, storage: GlobalsObject) -> &mut GlobalsObject {
        let mut globals_lock = self.globals.lock().unwrap();
        globals_lock.push(storage);
        &mut globals_lock.get_last_segments_ref()[0]
    }

    // unsafe: extracts mut-ref on cluster.tables without owning cluster.tables
    pub(crate) fn alloc_table_items(&self) -> &mut TableObject {
        let mut tables_lock = self.tables.lock().unwrap();
        tables_lock.push(TableObject(Vec::new()));
        &mut tables_lock.get_last_segments_ref()[0]
    }

    // unsafe: extracts mut-ref on cluster.memories without owning cluster.memories
    pub(crate) fn alloc_memories(&self, memories: Vec<MemoryObject>) -> &mut [MemoryObject] {
        let mut memories_lock = self.memories.lock().unwrap();
        memories_lock.extend(memories);
        memories_lock.get_last_segments_ref()
    }

    pub(crate) fn alloc_execution_context(
        &self,
        mut execution_context: ExecutionContext,
    ) -> &mut ExecutionContext {
        let mut execution_contexts_lock = self.execution_contexts.lock().unwrap();
        execution_context.id = execution_contexts_lock.len() as u32;
        execution_contexts_lock.push(execution_context);
        &mut execution_contexts_lock.get_last_segments_ref()[0]
    }

    pub(crate) fn alloc_engine(&self, engine: Engine) -> &mut Engine {
        let mut engines_lock = self.engines.lock().unwrap();
        engines_lock.push(engine);
        &mut engines_lock.get_last_segments_ref()[0]
    }

    pub(crate) fn alloc_function(&self, function: Function) -> &mut Function {
        let mut functions_lock = self.functions.lock().unwrap();
        functions_lock.push(function);
        &mut functions_lock.get_last_segments_ref()[0]
    }

    pub(crate) fn alloc_wasi_context(&self, wasi_ctxt: WasiContext) -> &mut WasiContext {
        let mut wasi_ctxt_lock = self.wasi_ctxt.lock().unwrap();
        wasi_ctxt_lock.push(wasi_ctxt);
        &mut wasi_ctxt_lock.get_last_segments_ref()[0]
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
            config: ClusterConfig::default(),
            memories: Mutex::new(SegmentedList::new()),
            tables: Mutex::new(SegmentedList::new()),
            globals: Mutex::new(SegmentedList::new()),
            execution_contexts: Mutex::new(SegmentedList::new()),
            engines: Mutex::new(SegmentedList::new()),
            functions: Mutex::new(SegmentedList::new()),
            wasi_ctxt: Mutex::new(SegmentedList::new()),
        }
    }
}
