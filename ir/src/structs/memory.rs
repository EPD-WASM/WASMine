use thiserror::Error;

use wasm_types::Limits;

const PAGE_SIZE: u32 = 65536; // 64 KiB, 2^16B

#[derive(Error, Debug, Clone)]
pub enum MemoryError {
    #[error("Index {index} out of range for limits {limits:?}")]
    OutOfRangeError { index: usize, limits: Limits },
}

#[derive(Debug, Clone)]
pub struct Memory {
    pub limits: Limits,
    data: Vec<u8>,
}

trait Memorylike {
    fn load_bytes(&self, offset: u32, size: u32) -> Option<Vec<u8>>;
    fn store_bytes(&mut self, offset: u32, data: &[u8]) -> Result<(), MemoryError>;
}

impl Memory {
    pub fn new(limits: Limits) -> Self {
        let size = (limits.min * PAGE_SIZE) as usize;
        Self {
            limits,
            data: vec![0; size],
        }
    }
}

impl Memorylike for Memory {
    fn load_bytes(&self, offset: u32, size: u32) -> Option<Vec<u8>> {
        let offset = offset as usize;
        let size = size as usize;
        self.data.get(offset..offset + size).map(Vec::from)
    }

    fn store_bytes(&mut self, offset: u32, data: &[u8]) -> Result<(), MemoryError> {
        let offset = offset as usize;

        if (offset + data.len()) < (self.limits.min * PAGE_SIZE) as usize {
            return Err(MemoryError::OutOfRangeError {
                index: offset,
                limits: self.limits.clone(),
            });
        }
        if let Some(max) = self.limits.max {
            if (offset + data.len()) > (max * PAGE_SIZE) as usize {
                return Err(MemoryError::OutOfRangeError {
                    index: offset,
                    limits: self.limits.clone(),
                });
            }
        }

        if (offset + data.len()) > self.data.len() {
            let new_num_pages = (offset + data.len()) % PAGE_SIZE as usize + 1; // +1 to round up, this could result in an unnecessary extra page
            self.data.resize(new_num_pages * PAGE_SIZE as usize, 0);
        }

        self.data[offset..offset + data.len()].copy_from_slice(data);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct MemArg {
    pub offset: u32,
    pub align: u32,
}
