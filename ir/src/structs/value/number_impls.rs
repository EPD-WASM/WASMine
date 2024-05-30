use crate::utils::numeric_transmutes::{Bit32, Bit64};
use wasm_types::NumType;

use super::Number;

impl Number {
    pub fn trans_to_u64(&self) -> u64 {
        match *self {
            Number::U32(n) => n as u64,
            Number::U64(n) => n,
            Number::S32(n) => n.trans_u64(),
            Number::S64(n) => n.trans_u64(),
            Number::F32(n) => n.trans_u64(),
            Number::F64(n) => n.trans_u64(),
            Number::I32(n) => n as u64,
            Number::I64(n) => n,
        }
    }

    pub fn trans_from_u64(n: u64, t: &NumType) -> Number {
        match t {
            // TODO: the spec stores I32s as signed, but this should't make a difference
            NumType::I32 => Self::U32(n as u32),
            NumType::I64 => Self::U64(n),
            NumType::F32 => Self::F32(n.trans_f32()),
            NumType::F64 => Self::F64(n.trans_f64()),
        }
    }

    pub fn as_signed(&self) -> Number {
        match *self {
            // TODO: this must be transmuted. range(i32) != range(u32)
            Number::U32(n) => Number::S32(n as i32),
            // TODO: this must be transmuted. range(i64) != range(u64)
            Number::U64(n) => Number::S64(n as i64),
            Number::F32(_) | Number::F64(_) => panic!("Invalid type for as_signed"),
            _ => self.clone(),
        }
    }

    pub fn as_unsigned(&self) -> Number {
        match *self {
            // TODO: same here, this should be transmuted
            Number::S32(n) => Number::U32(n as u32),
            Number::S64(n) => Number::U64(n as u64),
            Number::F32(_) | Number::F64(_) => panic!("Invalid type for as_unsigned"),
            _ => self.clone(),
        }
    }

    pub fn rotate_left(&self, n: Number) -> Number {
        match (self, n) {
            (Number::U32(a), Number::U32(b)) => Number::U32(a.rotate_left(b)),
            (Number::U64(a), Number::U64(b)) => Number::U64(a.rotate_left(b as u32)),
            (Number::S32(a), Number::U32(b)) => Number::S32(a.rotate_left(b)),
            (Number::S64(a), Number::U64(b)) => Number::S64(a.rotate_left(b as u32)),
            _ => panic!("Type mismatch"),
        }
    }

    pub fn rotate_right(&self, n: Number) -> Number {
        match (self, n) {
            (Number::U32(a), Number::U32(b)) => Number::U32(a.rotate_right(b)),
            (Number::U64(a), Number::U64(b)) => Number::U64(a.rotate_right(b as u32)),
            (Number::S32(a), Number::U32(b)) => Number::S32(a.rotate_right(b)),
            (Number::S64(a), Number::U64(b)) => Number::S64(a.rotate_right(b as u32)),
            _ => panic!("Type mismatch"),
        }
    }

    pub fn abs(&self) -> Number {
        match *self {
            Number::S32(n) => Number::S32(n.abs()),
            Number::S64(n) => Number::S64(n.abs()),
            Number::F32(n) => Number::F32(n.abs()),
            Number::F64(n) => Number::F64(n.abs()),
            _ => self.clone(),
        }
    }

    pub fn sqrt(&self) -> Number {
        match *self {
            Number::F32(n) => Number::F32(n.sqrt()),
            Number::F64(n) => Number::F64(n.sqrt()),
            _ => panic!("Invalid type for sqrt"),
        }
    }

    pub fn ceil(&self) -> Number {
        match *self {
            Number::F32(n) => Number::F32(n.ceil()),
            Number::F64(n) => Number::F64(n.ceil()),
            _ => panic!("Invalid type for ceil"),
        }
    }

    pub fn floor(&self) -> Number {
        match *self {
            Number::F32(n) => Number::F32(n.floor()),
            Number::F64(n) => Number::F64(n.floor()),
            _ => panic!("Invalid type for floor"),
        }
    }

    pub fn trunc(&self) -> Number {
        match *self {
            Number::F32(n) => Number::F32(n.trunc()),
            Number::F64(n) => Number::F64(n.trunc()),
            _ => panic!("Invalid type for trunc"),
        }
    }

    pub fn nearest(&self) -> Number {
        match *self {
            Number::F32(n) => Number::F32(n.round()),
            Number::F64(n) => Number::F64(n.round()),
            _ => panic!("Invalid type for nearest"),
        }
    }

    pub fn min(&self, other: &Number) -> Number {
        match (self, other) {
            (Number::U32(a), Number::U32(b)) => Number::U32((*a).min(*b)),
            (Number::U64(a), Number::U64(b)) => Number::U64((*a).min(*b)),
            (Number::S32(a), Number::S32(b)) => Number::S32((*a).min(*b)),
            (Number::S64(a), Number::S64(b)) => Number::S64((*a).min(*b)),
            (Number::F32(a), Number::F32(b)) => Number::F32((*a).min(*b)),
            (Number::F64(a), Number::F64(b)) => Number::F64((*a).min(*b)),
            _ => panic!("Type mismatch"),
        }
    }

    pub fn max(&self, other: &Number) -> Number {
        match (self, other) {
            (Number::U32(a), Number::U32(b)) => Number::U32((*a).max(*b)),
            (Number::U64(a), Number::U64(b)) => Number::U64((*a).max(*b)),
            (Number::S32(a), Number::S32(b)) => Number::S32((*a).max(*b)),
            (Number::S64(a), Number::S64(b)) => Number::S64((*a).max(*b)),
            (Number::F32(a), Number::F32(b)) => Number::F32((*a).max(*b)),
            (Number::F64(a), Number::F64(b)) => Number::F64((*a).max(*b)),
            _ => panic!("Type mismatch"),
        }
    }

    pub fn copysign(&self, other: &Number) -> Number {
        match (self, other) {
            (Number::F32(a), Number::F32(b)) => Number::F32(a.copysign(*b)),
            (Number::F64(a), Number::F64(b)) => Number::F64(a.copysign(*b)),
            _ => panic!("Type mismatch"),
        }
    }
}
