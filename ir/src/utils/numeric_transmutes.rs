use std::mem::transmute;

use crate::structs::value::Number;
use wasm_types::NumType;

pub trait Bit32 {
    fn trans_u32(&self) -> u32;
    fn trans_i32(&self) -> i32;
    fn trans_f32(&self) -> f32;

    fn trans_u64(&self) -> u64 {
        self.trans_u32() as u64
    }
}
pub trait Bit64 {
    fn trans_u64(&self) -> u64;
    fn trans_i64(&self) -> i64;
    fn trans_f64(&self) -> f64;

    // "Casting from a larger integer to a smaller integer (e.g. u32 -> u8) will truncate"
    fn trans_u32(&self) -> u32 {
        self.trans_u64() as u32
    }

    // "Casting from a larger integer to a smaller integer (e.g. u32 -> u8) will truncate"
    fn trans_i32(&self) -> i32 {
        self.trans_i64() as i32
    }

    fn trans_f32(&self) -> f32 {
        let n_u32 = self.trans_u64() as u32;
        f32::from_bits(n_u32)
    }

    fn to_number(&self, t: &NumType) -> Number {
        match t {
            NumType::I32 => Number::U32(self.trans_u32()),
            NumType::I64 => Number::U64(self.trans_u64()),
            NumType::F32 => Number::F32(self.trans_f32()),
            NumType::F64 => Number::F64(self.trans_f64()),
        }
    }
}

impl Bit32 for u32 {
    fn trans_u32(&self) -> u32 {
        *self
    }

    fn trans_i32(&self) -> i32 {
        unsafe { transmute::<u32, i32>(*self) }
    }

    fn trans_f32(&self) -> f32 {
        f32::from_bits(*self)
    }
}

impl Bit32 for i32 {
    fn trans_u32(&self) -> u32 {
        unsafe { transmute::<i32, u32>(*self) }
    }

    fn trans_i32(&self) -> i32 {
        *self
    }

    fn trans_f32(&self) -> f32 {
        f32::from_bits(self.trans_u32())
    }
}

impl Bit32 for f32 {
    fn trans_u32(&self) -> u32 {
        self.to_bits()
    }

    fn trans_i32(&self) -> i32 {
        self.to_bits().trans_i32()
    }

    fn trans_f32(&self) -> f32 {
        *self
    }
}

impl Bit64 for u64 {
    fn trans_u64(&self) -> u64 {
        *self
    }

    fn trans_i64(&self) -> i64 {
        unsafe { transmute::<u64, i64>(*self) }
    }

    fn trans_f64(&self) -> f64 {
        f64::from_bits(*self)
    }
}

impl Bit64 for i64 {
    fn trans_u64(&self) -> u64 {
        unsafe { transmute::<i64, u64>(*self) }
    }

    fn trans_i64(&self) -> i64 {
        *self
    }

    fn trans_f64(&self) -> f64 {
        f64::from_bits(self.trans_u64())
    }
}

impl Bit64 for f64 {
    fn trans_u64(&self) -> u64 {
        self.to_bits()
    }

    fn trans_i64(&self) -> i64 {
        self.to_bits().trans_i64()
    }

    fn trans_f64(&self) -> f64 {
        *self
    }
}
