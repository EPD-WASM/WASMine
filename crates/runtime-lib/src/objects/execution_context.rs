use crate::error::RuntimeError;
use crate::objects::tables::{TableError, TableInstance, TableItem};
use crate::Engine;
use cee_scape::SigJmpBuf;
use core::slice;
use runtime_interface::RawPointer;
use std::cell::RefCell;
use std::fmt::Display;
use std::ptr::null;
use wasm_types::{TableIdx, TypeIdx};

thread_local! {
    static TRAP_RETURN: RefCell<SigJmpBuf> = const { RefCell::new(null()) };
    static TRAP_ERR: RefCell<RuntimeError> = const { RefCell::new(RuntimeError::None) };
}

#[repr(transparent)]
pub(crate) struct ExecutionContextWrapper<'a>(
    pub(crate) &'a mut runtime_interface::ExecutionContext,
);

impl ExecutionContextWrapper<'_> {
    pub(crate) fn trap(e: RuntimeError) -> ! {
        TRAP_ERR.replace(e);
        TRAP_RETURN.with(|buf| unsafe { cee_scape::siglongjmp(*buf.as_ptr(), 1) })
    }

    pub(crate) fn take_trap() -> RuntimeError {
        TRAP_ERR.take()
    }

    pub(crate) fn get_tables(&mut self) -> &mut [TableInstance] {
        unsafe {
            slice::from_raw_parts_mut(self.0.tables_ptr as *mut TableInstance, self.0.tables_len)
        }
    }

    pub(crate) fn set_trap_return_point(buf: SigJmpBuf) {
        TRAP_RETURN.replace(buf);
    }

    pub(crate) fn indirect_call(
        &mut self,
        table_idx: TableIdx,
        entry_idx: u32,
        ty_idx: TypeIdx,
    ) -> Result<RawPointer, RuntimeError> {
        let engine = unsafe { &mut *(self.0.engine as *mut Engine) };
        let wasm_module = self.0.wasm_module.clone();
        let tables = self.get_tables();
        let table = &mut tables[table_idx as usize];
        if entry_idx >= table.size() {
            return Err(TableError::TableAccessOutOfBounds.into());
        }
        let reference = &mut table.values.0[entry_idx as usize];
        match reference {
            TableItem::FunctionReference {
                func_ptr,
                func_idx,
                func_type,
            } => {
                let expected_function_type = &wasm_module.meta.function_types[ty_idx as usize];
                let actual_function_type = &wasm_module.meta.function_types[*func_type as usize];
                if expected_function_type != actual_function_type {
                    return Err(TableError::TableFunctionTypeMismatch {
                        expected: *expected_function_type,
                        actual: *actual_function_type,
                    }
                    .into());
                }

                if func_ptr.is_none() {
                    *func_ptr = Some(engine.get_internal_function_ptr(*func_idx)?);
                }
                Ok(func_ptr.unwrap())
            }
            TableItem::ExternReference { func_ptr } => Ok(func_ptr.unwrap()),
            TableItem::Null => Err(TableError::NullDeref.into()),
        }
    }
}

pub(crate) fn trap_on_err<R, E>(
    ctxt: &mut runtime_interface::ExecutionContext,
    res: Result<R, E>,
) -> R
where
    E: Into<RuntimeError> + Display,
{
    match res {
        Ok(r) => r,
        Err(e) => ExecutionContextWrapper::trap(e.into()),
    }
}
