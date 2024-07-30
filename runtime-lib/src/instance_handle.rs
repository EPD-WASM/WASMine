use crate::{
    engine::Engine,
    func::{Function, FunctionKind},
    globals::GlobalsObject,
    linker::{rt_func_imports, DependencyStore},
    memory::{MemoryError, MemoryObject, MemoryStorage},
    signals::SignalHandler,
    tables::{TableError, TableInstance},
    utils::Either,
    Cluster, RuntimeError,
};
use core::{ffi, slice};
use ir::structs::{module::Module as WasmModule, value::Value};
use runtime_interface::{ExecutionContext, GlobalInstance};
use std::{collections::HashMap, ptr::null_mut, rc::Rc, sync::Mutex};
use wasm_types::{FuncIdx, FuncType, GlobalIdx, MemIdx, TableIdx};

#[derive(thiserror::Error, Debug)]
pub enum InstantiationError {
    #[error("Module does not contain start function.")]
    NoStartFunction,
    #[error("Error during table initialization: {0}")]
    TableError(#[from] TableError),
    #[error("Error during memory initialization: {0}")]
    MemoryError(#[from] MemoryError),
    #[error("Function not found: {0}")]
    FunctionNotFound(String),
}

pub struct InstanceHandle<'a> {
    pub(crate) cluster: &'a Cluster,

    module: Rc<WasmModule>,
    engine: &'a mut Engine,
    execution_context: &'a mut ExecutionContext,

    globals: &'a mut GlobalsObject,
    tables: Vec<TableInstance<'a>>,
    memories: &'a mut [MemoryObject],

    exported_functions: Mutex<HashMap<String, Either<&'a Function, FuncIdx>>>,
}

impl<'a> InstanceHandle<'a> {
    pub(crate) fn new(
        cluster: &'a Cluster,
        m: Rc<WasmModule>,
        engine: Engine,
        imports: DependencyStore,
    ) -> Result<Self, InstantiationError> {
        let engine = cluster.alloc_engine(engine);
        let execution_context = cluster.alloc_execution_context(ExecutionContext {
            tables_ptr: null_mut(),
            tables_len: 0,
            globals_ptr: null_mut(),
            globals_len: 0,
            memories_ptr: null_mut(),
            memories_len: 0,
            wasm_module: m.clone(),
            engine: engine as *mut Engine as *mut ffi::c_void,
            trap_msg: None,
            recursion_size: 0,
            id: 0,
        });

        for f in imports.functions.iter() {
            let (ptr, ctxt) = match f.func.kind {
                FunctionKind::Host(ptr, ctxt) => (ptr, ctxt),
                FunctionKind::Wasm(ptr, wasm_func_ptr, ctxt) => {
                    engine.register_symbol(
                        &(f.name.symbol_name() + "_wasm_cc"),
                        wasm_func_ptr.as_ptr(),
                    );
                    (ptr, ctxt)
                }
                _ => panic!("unexpected function kind"),
            };
            engine.register_symbol(&f.name.symbol_name(), ptr as _);
            engine.register_symbol(&format!("import_{}_rt_ptr", f.name.module), unsafe {
                ctxt.execution_context as _
            });
        }

        for import in rt_func_imports(execution_context as _) {
            let ptr = match import.func.kind {
                FunctionKind::Runtime(ptr) => ptr,
                _ => panic!("unexpected function kind"),
            };
            engine.register_symbol(&import.name.symbol_name(), ptr.as_ptr())
        }

        // initialize globals
        let globals =
            GlobalsObject::init_on_cluster(cluster, &m.globals, &imports.globals, engine)?;
        execution_context.globals_ptr = &mut globals.inner as *mut runtime_interface::GlobalStorage;
        execution_context.globals_len = 1;

        // initialize tables
        let tables = Self::init_tables_on_cluster(
            m.as_ref(),
            engine,
            cluster,
            &m.tables,
            &m.elements,
            &imports.tables,
            &globals.inner,
        )?;
        execution_context.tables_ptr = tables.as_ptr() as *mut ffi::c_void;
        execution_context.tables_len = tables.len();

        // initialize memories
        let memories = MemoryStorage::init_on_cluster(
            cluster,
            &m.memories,
            &m.datas,
            &imports.memories,
            &globals.inner,
        )?;
        execution_context.memories_ptr =
            memories.as_mut_ptr() as *mut runtime_interface::MemoryInstance;
        execution_context.memories_len = memories.len();

        let exported_functions = Mutex::new(
            m.exports
                .functions()
                .map(|(name, idx)| (name.clone(), Either::Right(*idx)))
                .collect(),
        );

        SignalHandler::register_globally();
        Ok(Self {
            module: m.clone(),
            engine,
            execution_context,
            globals,
            tables,
            memories,
            cluster,
            exported_functions,
        })
    }

    pub fn query_start_function(&self) -> Result<FuncIdx, InstantiationError> {
        if let Some(start_func_idx) = self.module.entry_point {
            return Ok(start_func_idx);
        }
        self.find_exported_func_idx("_start")
            .or_else(|_| self.find_exported_func_idx("run"))
            .or(Err(InstantiationError::NoStartFunction))
    }

    pub fn find_exported_func_idx(&self, name: &str) -> Result<FuncIdx, InstantiationError> {
        if let Some(entry) = self.module.entry_point {
            if name == format!("func_{}", entry) {
                return Ok(entry);
            }
        }
        self.module
            .exports
            .find_function_idx(name)
            .ok_or(InstantiationError::FunctionNotFound(name.to_string()))
    }

    pub fn get_function_type_from_func_idx(&self, func_idx: u32) -> &FuncType {
        &self.module.function_types[self.module.ir.functions[func_idx as usize].type_idx as usize]
    }

    pub fn get_function_type_from_name(&self, name: &str) -> Option<&FuncType> {
        self.module.exports.find_function_idx(name).map(|idx| {
            &self.module.function_types[self.module.ir.functions[idx as usize].type_idx as usize]
        })
    }

    pub fn get_function(&self, name: &str) -> Result<&Function, RuntimeError> {
        let mut locked_exported_functions = self.exported_functions.lock().unwrap();
        if let Some(function_entry) = locked_exported_functions.get(name) {
            match function_entry {
                Either::Left(func) => return Ok(*func),
                Either::Right(func_idx) => {
                    let idx = match self.wasm_module().exports.find_function_idx(name) {
                        Some(idx) => idx,
                        None => return Err(RuntimeError::FunctionNotFound(name.to_string())),
                    };
                    let func = Function::from_wasm_func(
                        self.execution_context_ptr(),
                        self.get_function_type_from_func_idx(*func_idx).clone(),
                        self.engine.get_external_function_ptr(idx)?,
                        self.engine.get_internal_function_ptr(idx)?,
                    );
                    let func_ref = self.cluster.alloc_function(func);
                    locked_exported_functions.insert(name.to_string(), Either::Left(func_ref));
                    return Ok(func_ref);
                }
            }
        }
        Err(RuntimeError::FunctionNotFound(name.to_string()))
    }

    #[inline]
    pub fn wasm_module(&self) -> &Rc<WasmModule> {
        &self.module
    }

    pub fn extract_global_value_by_idx(&self, idx: usize) -> Value {
        let global_addr = self.globals.inner.globals[idx].addr;
        let global_valty = match &self.module.globals[idx].r#type {
            wasm_types::GlobalType::Const(ty) => ty,
            wasm_types::GlobalType::Mut(ty) => ty,
        };
        let global_val = unsafe { *(global_addr) };
        Value::from_raw(global_val, *global_valty)
    }

    pub fn extract_global_value_by_name(&self, name: &str) -> Option<Value> {
        self.module
            .exports
            .find_global_idx(name)
            .map(|i| self.extract_global_value_by_idx(i as usize))
    }

    pub(crate) fn memories(&self, mem_idx: MemIdx) -> &MemoryObject {
        &self.memories[mem_idx as usize]
    }

    pub(crate) fn globals(&self, global_idx: GlobalIdx) -> &GlobalInstance {
        &self.globals.inner.globals[global_idx as usize]
    }

    pub(crate) fn tables(&self, table_idx: TableIdx) -> &TableInstance {
        &self.tables[table_idx as usize]
    }

    pub(crate) fn execution_context_ref(&self) -> &ExecutionContext {
        self.execution_context
    }

    pub(crate) fn execution_context_ptr(&self) -> *mut ExecutionContext {
        self.execution_context as *const _ as *mut _
    }

    pub(crate) fn engine(&mut self) -> &mut Engine {
        self.engine
    }

    pub fn run_by_name(
        &mut self,
        func_name: &str,
        input_params: Vec<Value>,
    ) -> Result<Vec<Value>, RuntimeError> {
        let func: &Function = self.get_function(func_name)?;
        func.call(&input_params)
    }
}

impl<'a> Clone for InstanceHandle<'a> {
    // even though this looks bad, it is merely here to fool the borrow checker again. All the contained references are unsafe and non-owning anyways.
    // Sole owner of all referenced objects is the cluster object with lifetime 'a.
    fn clone(&self) -> Self {
        SignalHandler::register_globally();
        unsafe {
            Self {
                module: self.module.clone(),
                cluster: &slice::from_raw_parts(self.cluster as *const Cluster, 1)[0],

                engine: &mut slice::from_raw_parts_mut(
                    self.engine as *const Engine as *mut Engine,
                    1,
                )[0],
                execution_context: &mut slice::from_raw_parts_mut(
                    self.execution_context as *const _ as *mut _,
                    1,
                )[0],

                globals: &mut slice::from_raw_parts_mut(self.globals as *const _ as *mut _, 1)[0],
                tables: self
                    .tables
                    .iter()
                    .map(|t| TableInstance {
                        values: &mut slice::from_raw_parts_mut(t.values as *const _ as *mut _, 1)
                            [0],
                        ty: t.ty,
                    })
                    .collect(),
                memories: slice::from_raw_parts_mut(
                    self.memories.as_ptr() as *mut _,
                    self.memories.len(),
                ),

                exported_functions: Mutex::new(self.exported_functions.lock().unwrap().clone()),
            }
        }
    }
}

impl Drop for InstanceHandle<'_> {
    fn drop(&mut self) {
        SignalHandler::deregister_globally();
    }
}
