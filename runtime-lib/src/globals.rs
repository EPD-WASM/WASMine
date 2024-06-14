use ir::structs::global::Global;
use nix::errno::Errno;
use runtime_interface::{GlobalInstance, GlobalStorage};

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

    GlobalStorage {
        storage: storage as *mut u8,
        globals,
    }
}
