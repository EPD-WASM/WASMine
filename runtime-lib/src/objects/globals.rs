use super::instance_handle::InstantiationError;
use crate::{linker::RTGlobalImport, Cluster, Engine};
use ir::structs::{
    global::Global,
    value::{ConstantValue, ValueRaw},
};
use nix::errno::Errno;
use runtime_interface::GlobalInstance;
use std::ptr::null_mut;

pub struct GlobalsObject {
    pub(crate) inner: runtime_interface::GlobalStorage,
}

impl GlobalsObject {
    pub(crate) fn init_on_cluster<'a>(
        cluster: &'a Cluster,
        globals_meta: &[Global],
        imports: &[RTGlobalImport],
        engine: &mut Engine,
    ) -> Result<&'a mut GlobalsObject, InstantiationError> {
        let storage_size = globals_meta.len() * 8;
        let storage = if storage_size > 0 {
            unsafe {
                libc::mmap(
                    std::ptr::null_mut(),
                    storage_size,
                    libc::PROT_READ | libc::PROT_WRITE,
                    libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
                    -1,
                    0,
                )
            }
        } else {
            std::ptr::null_mut()
        };
        if storage == libc::MAP_FAILED {
            panic!("Failed to allocate global storage: {}", Errno::last());
        }

        let mut globals = vec![GlobalInstance { addr: null_mut() }; globals_meta.len()];
        for instance in imports.iter().rev() {
            globals[instance.idx as usize].addr = instance.addr;
            engine.register_symbol(
                &format!("__wasmine_global__{}", instance.idx),
                globals[instance.idx as usize].addr as _,
            );
        }
        for (idx, global) in globals_meta.iter().enumerate() {
            if !global.import {
                let addr = unsafe { (storage as *mut ValueRaw).add(idx) };
                globals[idx].addr = addr;
                engine.register_symbol(&format!("__wasmine_global__{}", idx), addr as _);
            }
        }
        for (idx, global) in globals_meta.iter().enumerate() {
            if !global.import {
                unsafe {
                    *globals[idx].addr = match global.init.clone() {
                        ConstantValue::V(value) => value.into(),
                        ConstantValue::Global(glob_idx) => *globals[glob_idx as usize].addr,
                        ConstantValue::FuncPtr(func_idx) => ValueRaw::funcref(func_idx),
                    }
                }
            }
        }

        let storage = GlobalsObject {
            inner: runtime_interface::GlobalStorage {
                storage: storage as *mut u8,
                globals,
            },
        };
        Ok(cluster.alloc_global_storage(storage))
    }
}

impl Drop for GlobalsObject {
    fn drop(&mut self) {
        if self.inner.storage.is_null() {
            return;
        }
        unsafe {
            libc::munmap(
                self.inner.storage as *mut libc::c_void,
                self.inner.globals.len() * 8,
            );
        }
    }
}
