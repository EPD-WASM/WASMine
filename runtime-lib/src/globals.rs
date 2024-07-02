use ir::{
    structs::{
        global::Global,
        value::{Number, Value},
    },
    utils::numeric_transmutes::{Bit32, Bit64},
};
use nix::errno::Errno;
use runtime_interface::GlobalInstance;

pub struct GlobalStorage {
    pub(crate) inner: runtime_interface::GlobalStorage,
    uninitialized_imports: Vec<usize>,
}

impl GlobalStorage {
    pub(crate) fn new(globals_meta: &[Global]) -> GlobalStorage {
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
        let globals = globals_meta
            .iter()
            .enumerate()
            .map(|(idx, global)| {
                let addr = unsafe { (storage as *mut u64).add(idx) };
                match global.import {
                    true => uninitialized_imports.push(idx),
                    false => unsafe {
                        *addr = match &global.init {
                            Value::Number(Number::I32(n)) => n.trans_u64(),
                            Value::Number(Number::I64(n)) => n.trans_u64(),
                            Value::Number(Number::F32(n)) => n.trans_u64(),
                            Value::Number(Number::F64(n)) => n.trans_u64(),
                            _ => unimplemented!(),
                        }
                    },
                }
                GlobalInstance { addr }
            })
            .collect();

        GlobalStorage {
            inner: runtime_interface::GlobalStorage {
                storage: storage as *mut u8,
                globals,
            },
            uninitialized_imports: Vec::new(),
        }
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
