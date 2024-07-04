use crate::{linker::RTImport, module_instance::InstantiationError, Cluster};
use ir::structs::global::Global;
use nix::errno::Errno;
use runtime_interface::GlobalInstance;

pub struct GlobalStorage {
    pub(crate) inner: runtime_interface::GlobalStorage,
    uninitialized_imports: Vec<usize>,
}

impl GlobalStorage {
    pub(crate) fn init_on_cluster<'a>(
        cluster: &'a Cluster,
        globals_meta: &[Global],
        imports: &[RTImport],
    ) -> Result<&'a mut GlobalStorage, InstantiationError> {
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

        let mut uninitialized_imports = Vec::new();
        let mut globals = globals_meta
            .iter()
            .enumerate()
            .map(|(idx, global)| {
                let addr = unsafe { (storage as *mut u64).add(idx) };
                match global.import {
                    true => uninitialized_imports.push(idx),
                    false => unsafe { *addr = global.init.to_generic() },
                }
                GlobalInstance { addr }
            })
            .collect::<Vec<_>>();

        for import in imports.iter().filter(|i| matches!(i, RTImport::Global(_))) {
            match import {
                RTImport::Global(instance) => {
                    let idx = uninitialized_imports.pop().unwrap();
                    unsafe { *globals[idx].addr = instance.init.to_generic() };
                }
                _ => unreachable!(),
            }
        }

        let storage = GlobalStorage {
            inner: runtime_interface::GlobalStorage {
                storage: storage as *mut u8,
                globals,
            },
            uninitialized_imports: Vec::new(),
        };
        Ok(cluster.alloc_global_storage(storage))
    }

    pub(crate) fn import(&mut self, instance: GlobalInstance) {
        let idx = self.uninitialized_imports.pop().unwrap();
        self.inner.globals[idx] = instance;
    }
}

impl Drop for GlobalStorage {
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
