use core::f32;

use crate::utils::numeric_transmutes::{Bit32, Bit64};
use wasm_types::NumType;

use super::Number;

// https://webassembly.github.io/spec/core/exec/numerics.html#nan-propagation

fn canonicalise_nan_f32(n: f32) -> f32 {
    if n.is_nan() {
        // When the result of a floating-point operator other than , , or is a NaN, then its sign is non-deterministic
        let canonical_nan: u32 = 0x7fc00000;
        canonical_nan.trans_f32()
    } else {
        n
    }
}

fn canonicalise_nan_f64(n: f64) -> f64 {
    if n.is_nan() {
        // When the result of a floating-point operator other than , , or is a NaN, then its sign is non-deterministic
        let canonical_nan: u64 = 0x7ff8000000000000;
        canonical_nan.trans_f64()
    } else {
        n
    }
}

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
            NumType::I32 => Self::I32(n as u32),
            NumType::I64 => Self::I64(n),
            NumType::F32 => Self::F32(n.trans_f32()),
            NumType::F64 => Self::F64(n.trans_f64()),
        }
    }

    pub fn trans_from_u64_sign(n: u64, t: &NumType, sign: bool) -> Number {
        match t {
            // TODO: the spec stores I32s as signed, but this should't make a difference
            NumType::I32 => {
                if sign {
                    Self::S32(n as i32)
                } else {
                    Self::U32(n as u32)
                }
            }
            NumType::I64 => {
                if sign {
                    Self::S64(n as i64)
                } else {
                    Self::U64(n)
                }
            }
            NumType::F32 => Self::F32(n.trans_f32()),
            NumType::F64 => Self::F64(n.trans_f64()),
        }
    }

    pub fn as_signed(&self) -> Number {
        match *self {
            Number::U32(n) => Number::S32(n.trans_i32()),
            Number::U64(n) => Number::S64(n.trans_i64()),
            Number::I32(n) => Number::S32(n.trans_i32()),
            Number::I64(n) => Number::S64(n.trans_i64()),
            Number::F32(_) | Number::F64(_) => panic!("Invalid type for as_signed"),
            _ => self.clone(),
        }
    }

    pub fn as_unsigned(&self) -> Number {
        match *self {
            Number::S32(n) => Number::U32(n.trans_u32()),
            Number::S64(n) => Number::U64(n.trans_u64()),
            Number::I32(n) => Number::U32(n.trans_u32()),
            Number::I64(n) => Number::U64(n.trans_u64()),
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
            (Number::I32(a), Number::I32(b)) => Number::I32(a.rotate_left(b)),
            (Number::I64(a), Number::I64(b)) => Number::I64(a.rotate_left(b as u32)),
            n @ (Number::F32(_), Number::F32(_)) | n @ (Number::F64(_), Number::F64(_)) => {
                panic!("Invalid type for rotate_left: {:?} rol {:?}", n.0, n.1)
            }
            (a, b) => panic!("Type mismatch: {a:?} rol {b:?}"),
        }
    }

    pub fn rotate_right(&self, n: Number) -> Number {
        match (self, n) {
            (Number::U32(a), Number::U32(b)) => Number::U32(a.rotate_right(b)),
            (Number::U64(a), Number::U64(b)) => Number::U64(a.rotate_right(b as u32)),
            (Number::S32(a), Number::U32(b)) => Number::S32(a.rotate_right(b)),
            (Number::S64(a), Number::U64(b)) => Number::S64(a.rotate_right(b as u32)),
            (Number::I32(a), Number::I32(b)) => Number::I32(a.rotate_right(b)),
            (Number::I64(a), Number::I64(b)) => Number::I64(a.rotate_right(b as u32)),
            n @ (Number::F32(_), Number::F32(_)) | n @ (Number::F64(_), Number::F64(_)) => {
                panic!("Invalid type for rotate_right: {:?} ror {:?}", n.0, n.1)
            }
            (a, b) => panic!("Type mismatch: {a:?} ror {b:?}"),
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
            Number::F32(n) => Number::F32(n.round_ties_even()),
            Number::F64(n) => Number::F64(n.round_ties_even()),
            _ => panic!("Invalid type for nearest"),
        }
    }

    pub fn min(&self, other: &Number) -> Number {
        match (self, other) {
            (Number::U32(a), Number::U32(b)) => Number::U32((*a).min(*b)),
            (Number::U64(a), Number::U64(b)) => Number::U64((*a).min(*b)),
            (Number::S32(a), Number::S32(b)) => Number::S32((*a).min(*b)),
            (Number::S64(a), Number::S64(b)) => Number::S64((*a).min(*b)),
            (Number::F32(a), Number::F32(b)) => Number::F32(canonicalise_nan_f32((*a).min(*b))),
            (Number::F64(a), Number::F64(b)) => Number::F64(canonicalise_nan_f64((*a).min(*b))),
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

    pub fn convert_to_f32(&self) -> Number {
        match *self {
            Number::U32(n) => Number::F32(n as f32),
            Number::U64(n) => Number::F32(n as f32),
            Number::S32(n) => Number::F32(n as f32),
            Number::S64(n) => Number::F32(n as f32),
            Number::F32(n) => Number::F32(n),
            Number::F64(n) => Number::F32(n as f32),
            Number::I32(n) => Number::F32(n as f32),
            Number::I64(n) => Number::F32(n as f32),
        }
    }

    pub fn convert_to_f64(&self) -> Number {
        match *self {
            Number::U32(n) => Number::F64(n as f64),
            Number::U64(n) => Number::F64(n as f64),
            Number::S32(n) => Number::F64(n as f64),
            Number::S64(n) => Number::F64(n as f64),
            Number::F32(n) => Number::F64(n as f64),
            Number::F64(n) => Number::F64(n),
            Number::I32(n) => Number::F64(n as f64),
            Number::I64(n) => Number::F64(n as f64),
        }
    }

    pub fn is_nan(&self) -> bool {
        match self {
            Number::F32(n) => n.is_nan(),
            Number::F64(n) => n.is_nan(),
            a => panic!("Invalid type for is_nan: {a}"),
        }
    }

    pub fn is_infinite(&self) -> bool {
        match self {
            Number::F32(n) => n.is_infinite(),
            Number::F64(n) => n.is_infinite(),
            a => panic!("Invalid type for is_infinite: {a}"),
        }
    }

    // why: https://doc.rust-lang.org/std/primitive.f64.html#associatedconstant.NAN
    pub fn nan(t: &NumType) -> Number {
        match t {
            NumType::F64 => Number::trans_from_u64(0x7ff8000000000000_u64, t),
            NumType::F32 => Number::trans_from_u64(0x7fc00000_u64, t),
            _ => panic!("Invalid type for nan: {t}"),
        }
    }
}
