use ir::structs::global::Global;
use nix::errno::Errno;
use runtime_interface::GlobalInstance;

pub struct GlobalStorage(pub runtime_interface::GlobalStorage);

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

    let globals = globals_meta
        .iter()
        .enumerate()
        .map(|(idx, _)| {
            let addr = unsafe { storage.add(idx) as *mut u64 };
            GlobalInstance { addr }
        })
        .collect();

    GlobalStorage(runtime_interface::GlobalStorage {
        storage: storage as *mut u8,
        globals,
    })
}

impl Drop for GlobalStorage {
    fn drop(&mut self) {
        if self.0.storage.is_null() {
            return;
        }
        unsafe {
            libc::munmap(
                self.0.storage as *mut libc::c_void,
                self.0.globals.len() * 8,
            );
        }
    }
}
