use crate::WASM_PAGE_SIZE;

use super::{
    types::{Errno, Ptr},
    WasiContext,
};
use core::slice;

impl WasiContext {
    pub(crate) fn get_memory_slice<T>(&self, ptr: Ptr<T>, len: usize) -> Result<&mut [T], Errno> {
        let memory = unsafe { &*(*self.execution_context).memories_ptr.add(0) };
        if std::mem::size_of::<T>() * len + ptr.get() as usize
            > (memory.size * WASM_PAGE_SIZE) as usize
        {
            return Err(Errno::Inval);
        }
        Ok(
            unsafe {
                slice::from_raw_parts_mut(memory.data.add(ptr.get() as usize) as *mut T, len)
            },
        )
    }
}

pub(crate) fn errno(o: Result<(), Errno>) -> Errno {
    match o {
        Ok(_) => Errno::Success,
        Err(e) => {
            log::debug!("-> Error: {:?}", e);
            e
        }
    }
}
