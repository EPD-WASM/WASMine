use crate::{
    linker::RTTableImport,
    objects::{
        engine::EngineError,
        execution_context::{trap_on_err, ExecutionContextWrapper},
        instance_handle::InstantiationError,
    },
    Cluster, Engine, InstanceHandle, RuntimeError,
};
use core::slice;
use module::{
    objects::{
        element::{ElemMode, Element, ElementInit},
        module::Module as WasmModule,
        table::Table,
        value::{ConstantValue, Number, Reference, Value, ValueRaw},
    },
    utils::numeric_transmutes::Bit32,
};
use runtime_interface::{ExecutionContext, GlobalStorage, RawPointer};
use wasm_types::{ElemIdx, FuncIdx, FuncType, RefType, TableIdx, TableType, TypeIdx, ValType};

#[derive(Debug, thiserror::Error)]
pub enum TableError {
    #[error("Table index out of bounds")]
    TableIndexOutOfBounds,
    #[error("Table access out of bounds")]
    TableAccessOutOfBounds,
    #[error("Elem access out of bounds")]
    ElemAccessOutOfBounds,
    #[error("Table element type mismatch: {expected:?} != {actual:?}")]
    TableElementTypeMismatch { expected: RefType, actual: RefType },
    #[error("Table function type mismatch: {expected:?} != {actual:?}")]
    TableFunctionTypeMismatch {
        expected: FuncType,
        actual: FuncType,
    },
    #[error("Reference is null.")]
    NullDeref,
    #[error("Engine error: {0}")]
    EngineError(#[from] EngineError),
    #[error("Offset into segment was of invalid type '{0:}'")]
    InvalidOffsetType(ValType),
}

#[derive(Debug, Clone)]
pub(crate) enum TableItem {
    FunctionReference {
        func_idx: FuncIdx,
        func_type: TypeIdx,
        func_ptr: Option<RawPointer>,
    },
    ExternReference {
        func_ptr: Option<RawPointer>,
    },
    Null,
}

#[repr(transparent)]
pub struct TableObject(pub Vec<TableItem>);

// All other information like the table type, etc. are stored in the config
pub struct TableInstance<'a> {
    // final, evaluated references
    pub(crate) values: &'a mut TableObject,
    pub(crate) ty: TableType,
}

impl InstanceHandle<'_> {
    pub(crate) fn init_tables_on_cluster<'a>(
        wasm_module: &WasmModule,
        engine: &Engine,
        cluster: &'a Cluster,
        tables_meta: &[Table],
        elems_meta: &[Element],
        imports: &[RTTableImport],
        globals: &GlobalStorage,
    ) -> Result<Vec<TableInstance<'a>>, InstantiationError> {
        let mut import_tables_iter = imports.iter();
        let mut tables = tables_meta
            .iter()
            .map(|table| {
                let table_items = if table.import {
                    unsafe {
                        &mut slice::from_raw_parts_mut(
                            import_tables_iter.next().unwrap().instance_ref,
                            1,
                        )[0]
                    }
                } else {
                    let table_items = cluster.alloc_table_items();
                    table_items
                        .0
                        .resize(table.r#type.lim.min as usize, TableItem::Null);
                    table_items
                };
                TableInstance {
                    values: table_items,
                    ty: table.r#type,
                }
            })
            .collect::<Vec<_>>();

        for elem in elems_meta.iter() {
            if let ElemMode::Active { table, offset } = &elem.mode {
                if *table as usize >= tables.len() {
                    return Err(TableError::TableIndexOutOfBounds.into());
                }

                let n = match elem.init {
                    ElementInit::Final(ref values) => values.len(),
                    ElementInit::Unresolved(ref func_idxs) => func_idxs.len(),
                };
                let offset = match offset {
                    ConstantValue::V(Value::Number(Number::I32(offset))) => *offset,
                    ConstantValue::V(v) => {
                        return Err(TableError::InvalidOffsetType(v.r#type()).into())
                    }
                    ConstantValue::Global(idx) => unsafe {
                        globals.globals[*idx as usize].addr.as_ref().as_u32()
                    },
                    ConstantValue::FuncPtr(_) => unimplemented!(),
                };
                tables[*table as usize].init(wasm_module, engine, elem, 0, offset, n as u32)?
                // TODO: drop elem
            }
        }
        Ok(tables)
    }
}

impl TableInstance<'_> {
    pub(crate) fn set(
        &mut self,
        wasm_module: &WasmModule,
        value: u64,
        idx: u32,
    ) -> Result<(), TableError> {
        if idx >= self.values.0.len() as u32 {
            return Err(TableError::TableIndexOutOfBounds);
        }
        let value = ValueRaw::from(value);
        if value.as_externref() == ValueRaw::from(Value::Reference(Reference::Null)).as_externref()
        {
            self.values.0[idx as usize] = TableItem::Null;
            return Ok(());
        }
        match self.ty.ref_type {
            RefType::FunctionReference => {
                let func_idx = value.as_funcref();
                debug_assert!(func_idx < wasm_module.meta.functions.len() as u32);
                self.values.0[idx as usize] = TableItem::FunctionReference {
                    func_idx,
                    func_ptr: None,
                    func_type: wasm_module.meta.functions[func_idx as usize].type_idx,
                }
            }
            RefType::ExternReference => {
                self.values.0[idx as usize] = TableItem::ExternReference {
                    func_ptr: RawPointer::new(value.as_externref() as _),
                }
            }
        }
        Ok(())
    }

    pub(crate) fn get(&self, idx: u32) -> Result<Value, TableError> {
        if idx >= self.values.0.len() as u32 {
            return Err(TableError::TableIndexOutOfBounds);
        }
        match self.values.0[idx as usize] {
            TableItem::FunctionReference { func_idx, .. } => Ok(Value::funcref(func_idx)),
            TableItem::ExternReference { func_ptr } => Ok(Value::externref(
                func_ptr.map(|ptr| ptr.as_ptr() as u64).unwrap_or(0),
            )),
            TableItem::Null => Ok(Value::Reference(Reference::Null)),
        }
    }

    pub(crate) fn size(&self) -> u32 {
        self.values.0.len() as u32
    }

    pub(crate) fn grow(
        &mut self,
        wasm_module: &WasmModule,
        size: u32,
        value_to_fill: u64,
    ) -> Result<u32, TableError> {
        let err = (-1_i32).trans_u32();
        let old_len = self.values.0.len();
        if size == 0 {
            return Ok(old_len as u32);
        }

        let new_len = old_len + size as usize;
        if new_len > self.ty.lim.max.unwrap_or(u32::MAX) as usize {
            log::debug!("Called table.grow with size > max size. Ignoring.");
            return Ok(err);
        }
        if self.values.0.try_reserve_exact(size as usize).is_err() {
            log::debug!("Failed to reserve space for table.grow. Ignoring.");
            return Ok(err);
        }

        let value_to_fill = ValueRaw::from(value_to_fill);
        if value_to_fill.as_externref()
            == ValueRaw::from(Value::Reference(Reference::Null)).as_externref()
        {
            self.values.0.resize(new_len, TableItem::Null);
            return Ok(old_len as u32);
        } else {
            let table_value_to_fill = match self.ty.ref_type {
                RefType::FunctionReference => TableItem::FunctionReference {
                    func_idx: value_to_fill.as_funcref(),
                    func_ptr: None,
                    func_type: wasm_module.meta.functions[value_to_fill.as_funcref() as usize]
                        .type_idx,
                },
                RefType::ExternReference => TableItem::ExternReference {
                    func_ptr: RawPointer::new(value_to_fill.as_externref() as _),
                },
            };
            self.values.0.resize(new_len, table_value_to_fill);
        }
        Ok(old_len as u32)
    }

    pub(crate) fn fill(
        &mut self,
        wasm_module: &WasmModule,
        start: u32,
        len: u32,
        value: u64,
    ) -> Result<(), TableError> {
        if start + len > self.values.0.len() as u32 {
            return Err(TableError::TableAccessOutOfBounds);
        }
        if len == 0 {
            return Ok(());
        }

        let value = ValueRaw::from(value);
        let value_to_fill = if value.as_externref()
            == ValueRaw::from(Value::Reference(Reference::Null)).as_externref()
        {
            TableItem::Null
        } else {
            match self.ty.ref_type {
                RefType::FunctionReference => TableItem::FunctionReference {
                    func_idx: value.as_funcref(),
                    func_ptr: None,
                    func_type: wasm_module.meta.functions[value.as_funcref() as usize].type_idx,
                },
                RefType::ExternReference => TableItem::ExternReference {
                    func_ptr: RawPointer::new(value.as_externref() as _),
                },
            }
        };
        self.values.0[start as usize..(start + len) as usize].fill(value_to_fill);
        Ok(())
    }

    pub(crate) fn copy(
        &mut self,
        // be careful: this could also be the same as self
        src_table: *const TableInstance,
        src_start: u32,
        dst_start: u32,
        len: u32,
    ) -> Result<(), TableError> {
        let src_table = unsafe { &*src_table };
        if src_start + len > src_table.values.0.len() as u32 {
            return Err(TableError::TableAccessOutOfBounds);
        }
        if dst_start + len > self.values.0.len() as u32 {
            return Err(TableError::TableAccessOutOfBounds);
        }
        if len == 0 {
            return Ok(());
        }
        // we use memmove instead of memcpy to prevent any issues resulting from overlapping memory
        unsafe {
            std::ptr::copy(
                src_table.values.0.as_ptr().add(src_start as usize),
                self.values.0.as_mut_ptr().add(dst_start as usize),
                len as usize,
            )
        };
        Ok(())
    }

    pub(crate) fn init(
        &mut self,
        wasm_module: &WasmModule,
        engine: &Engine,
        elem: &Element,
        src_offset: u32,
        dst_offset: u32,
        len: u32,
    ) -> Result<(), TableError> {
        let elem_data_len = match elem.init {
            ElementInit::Final(ref values) => values.len() as u32,
            ElementInit::Unresolved(ref func_idxs) => func_idxs.len() as u32,
        };
        if src_offset + len > elem_data_len {
            return Err(TableError::ElemAccessOutOfBounds);
        }
        if dst_offset + len > self.values.0.len() as u32 {
            return Err(TableError::TableAccessOutOfBounds);
        }
        let end_idx = src_offset + len;

        match &elem.init {
            ElementInit::Final(values) => {
                for (i, val) in values[src_offset as usize..(src_offset + len) as usize]
                    .iter()
                    .cloned()
                    .enumerate()
                {
                    let val = match val {
                        ConstantValue::V(v) => v,
                        ConstantValue::Global(idx) => Value::from_raw(
                            engine.get_global_value(idx)?,
                            wasm_module.meta.globals[idx as usize].val_type(),
                        ),
                        ConstantValue::FuncPtr(func_idx) => Value::funcref(func_idx),
                    };
                    self.values.0[(dst_offset as usize) + i] = match val {
                        Value::Number(Number::I32(func_idx))
                        | Value::Reference(Reference::Function(func_idx)) => {
                            TableItem::FunctionReference {
                                func_idx,
                                func_ptr: None,
                                func_type: wasm_module.meta.functions[func_idx as usize].type_idx,
                            }
                        }
                        Value::Reference(Reference::Extern(func_ptr)) => {
                            TableItem::ExternReference {
                                func_ptr: RawPointer::new(func_ptr as _),
                            }
                        }
                        Value::Reference(Reference::Null) => TableItem::Null,
                        _ => unreachable!(),
                    }
                }
            }
            ElementInit::Unresolved(func_idxs) => {
                for (i, func_idx) in func_idxs[src_offset as usize..(src_offset + len) as usize]
                    .iter()
                    .cloned()
                    .enumerate()
                {
                    self.values.0[(dst_offset as usize) + i] = TableItem::FunctionReference {
                        func_idx,
                        func_ptr: None,
                        func_type: wasm_module.meta.functions[func_idx as usize].type_idx,
                    };
                }
            }
        };
        Ok(())
    }
}

fn indirect_call_impl(
    ctxt: &mut ExecutionContext,
    table_idx: TableIdx,
    ty_idx: TypeIdx,
    entry_idx: u32,
) -> Result<RawPointer, RuntimeError> {
    let mut ctxt = ExecutionContextWrapper(ctxt);
    ctxt.indirect_call(table_idx, entry_idx, ty_idx)
}

fn table_set_impl(
    ctxt: &mut ExecutionContext,
    table_idx: usize,
    value: u64,
    idx: u32,
) -> Result<(), TableError> {
    let mut ctxt = ExecutionContextWrapper(ctxt);
    let wasm_module = ctxt.0.wasm_module.clone();
    let tables = ctxt.get_tables();
    let table = &mut tables[table_idx];
    table.set(&wasm_module, value, idx)
}

fn table_get_impl(
    ctxt: &mut ExecutionContext,
    table_idx: usize,
    idx: u32,
) -> Result<u64, TableError> {
    let mut ctxt = ExecutionContextWrapper(ctxt);
    let tables = ctxt.get_tables();
    let table = &tables[table_idx];
    Ok(ValueRaw::from(table.get(idx)?).as_u64())
}

fn table_grow_impl(
    ctxt: &mut ExecutionContext,
    table_idx: TableIdx,
    size: u32,
    value_to_fill: u64,
) -> Result<u32, TableError> {
    let mut ctxt = ExecutionContextWrapper(ctxt);
    let wasm_module = ctxt.0.wasm_module.clone();
    let tables = ctxt.get_tables();
    let table = &mut tables[table_idx as usize];
    table.grow(&wasm_module, size, value_to_fill)
}

fn table_size_impl(ctxt: &mut ExecutionContext, table_idx: usize) -> Result<u32, TableError> {
    let mut ctxt = ExecutionContextWrapper(ctxt);
    let tables = ctxt.get_tables();
    let table = &tables[table_idx];
    Ok(table.size())
}

fn table_fill_impl(
    ctxt: &mut ExecutionContext,
    table_idx: TableIdx,
    start: u32,
    len: u32,
    value: u64,
) -> Result<(), TableError> {
    let mut ctxt = ExecutionContextWrapper(ctxt);
    let wasm_module = ctxt.0.wasm_module.clone();
    let tables = ctxt.get_tables();
    let table = &mut tables[table_idx as usize];
    table.fill(&wasm_module, start, len, value)
}

fn table_copy_impl(
    ctxt: &mut ExecutionContext,
    src_table_idx: TableIdx,
    dst_table_idx: TableIdx,
    src_start: u32,
    dst_start: u32,
    len: u32,
) -> Result<(), TableError> {
    let mut ctxt = ExecutionContextWrapper(ctxt);
    let tables = ctxt.get_tables();
    let src_table = &tables[src_table_idx as usize] as *const _;
    let dst_table = &mut tables[dst_table_idx as usize];
    dst_table.copy(src_table, src_start, dst_start, len)
}

fn table_init_impl(
    ctxt: &mut ExecutionContext,
    table_idx: TableIdx,
    elem_idx: ElemIdx,
    src_offset: u32,
    dst_offset: u32,
    len: u32,
) -> Result<(), TableError> {
    let execution_context_ptr = ctxt as *mut ExecutionContext;
    let mut ctxt = ExecutionContextWrapper(ctxt);
    let wasm_module = ctxt.0.wasm_module.clone();
    let tables = ctxt.get_tables();
    let table = &mut tables[table_idx as usize];
    unsafe {
        table.init(
            &(*execution_context_ptr).wasm_module,
            &*((*execution_context_ptr).engine as *mut Engine),
            &wasm_module.meta.elements[elem_idx as usize],
            src_offset,
            dst_offset,
            len,
        )
    }
}

fn elem_drop_impl(ctxt: &mut ExecutionContext, elem_idx: ElemIdx) -> Result<(), TableError> {
    if elem_idx as usize >= ctxt.wasm_module.meta.elements.len() {
        return Err(TableError::ElemAccessOutOfBounds);
    }
    // we don't remove elems
    Ok(())
}

#[no_mangle]
pub extern "C" fn indirect_call(
    ctxt: &mut ExecutionContext,
    table_idx: TableIdx,
    type_idx: TypeIdx,
    entry_idx: u32,
) -> RawPointer {
    let res = indirect_call_impl(ctxt, table_idx, type_idx, entry_idx);
    trap_on_err(ctxt, res)
}
#[no_mangle]
pub extern "C" fn table_set(ctxt: &mut ExecutionContext, table_idx: usize, value: u64, idx: u32) {
    let res = table_set_impl(ctxt, table_idx, value, idx);
    trap_on_err(ctxt, res)
}
#[no_mangle]
pub extern "C" fn table_get(ctxt: &mut ExecutionContext, table_idx: usize, idx: u32) -> u64 {
    let res = table_get_impl(ctxt, table_idx, idx);
    trap_on_err(ctxt, res)
}
#[no_mangle]
pub extern "C" fn table_grow(
    ctxt: &mut ExecutionContext,
    table_idx: TableIdx,
    size: u32,
    value_to_fill: u64,
) -> u32 {
    let res = table_grow_impl(ctxt, table_idx, size, value_to_fill);
    trap_on_err(ctxt, res)
}
#[no_mangle]
pub extern "C" fn table_size(ctxt: &mut ExecutionContext, table_idx: usize) -> u32 {
    let res = table_size_impl(ctxt, table_idx);
    trap_on_err(ctxt, res)
}
#[no_mangle]
pub extern "C" fn table_fill(
    ctxt: &mut ExecutionContext,
    table_idx: TableIdx,
    start: u32,
    len: u32,
    value: u64,
) {
    let res = table_fill_impl(ctxt, table_idx, start, len, value);
    trap_on_err(ctxt, res)
}
#[no_mangle]
pub extern "C" fn table_copy(
    ctxt: &mut ExecutionContext,
    src_table_idx: TableIdx,
    dst_table_idx: TableIdx,
    src_start: u32,
    dst_start: u32,
    len: u32,
) {
    let res = table_copy_impl(
        ctxt,
        src_table_idx,
        dst_table_idx,
        src_start,
        dst_start,
        len,
    );
    trap_on_err(ctxt, res)
}
#[no_mangle]
pub extern "C" fn table_init(
    ctxt: &mut ExecutionContext,
    table_idx: TableIdx,
    elem_idx: ElemIdx,
    src_offset: u32,
    dst_offset: u32,
    len: u32,
) {
    let res = table_init_impl(ctxt, table_idx, elem_idx, src_offset, dst_offset, len);
    trap_on_err(ctxt, res)
}
#[no_mangle]
pub extern "C" fn elem_drop(ctxt: &mut ExecutionContext, elem_idx: ElemIdx) {
    let res = elem_drop_impl(ctxt, elem_idx);
    trap_on_err(ctxt, res)
}
