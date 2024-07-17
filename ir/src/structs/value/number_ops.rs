use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Rem, Shl, Shr, Sub};

use super::Number;

impl Add for Number {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match (self, other) {
            (Number::I32(a), Number::I32(b)) => Number::I32(a.wrapping_add(b)),
            (Number::I64(a), Number::I64(b)) => Number::I64(a.wrapping_add(b)),
            (Number::U32(a), Number::U32(b)) => Number::U32(a.wrapping_add(b)),
            (Number::U64(a), Number::U64(b)) => Number::U64(a.wrapping_add(b)),
            (Number::S32(a), Number::S32(b)) => Number::S32(a.wrapping_add(b)),
            (Number::S64(a), Number::S64(b)) => Number::S64(a.wrapping_add(b)),
            (Number::F32(a), Number::F32(b)) => Number::F32(a + b),
            (Number::F64(a), Number::F64(b)) => Number::F64(a + b),
            (a, b) => panic!("Type mismatch: {:?} + {:?}", a, b),
        }
    }
}

impl Sub for Number {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        match (self, other) {
            (Number::I32(a), Number::I32(b)) => Number::I32(a.wrapping_sub(b)),
            (Number::I64(a), Number::I64(b)) => Number::I64(a.wrapping_sub(b)),
            (Number::U32(a), Number::U32(b)) => Number::U32(a.wrapping_sub(b)),
            (Number::U64(a), Number::U64(b)) => Number::U64(a.wrapping_sub(b)),
            (Number::S32(a), Number::S32(b)) => Number::S32(a.wrapping_sub(b)),
            (Number::S64(a), Number::S64(b)) => Number::S64(a.wrapping_sub(b)),
            (Number::F32(a), Number::F32(b)) => Number::F32(a - b),
            (Number::F64(a), Number::F64(b)) => Number::F64(a - b),
            (a, b) => panic!("Type mismatch: {:?} - {:?}", a, b),
        }
    }
}

impl Mul for Number {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        match (self, other) {
            (Number::I32(a), Number::I32(b)) => Number::I32(a.wrapping_mul(b)),
            (Number::I64(a), Number::I64(b)) => Number::I64(a.wrapping_mul(b)),
            (Number::U32(a), Number::U32(b)) => Number::U32(a.wrapping_mul(b)),
            (Number::U64(a), Number::U64(b)) => Number::U64(a.wrapping_mul(b)),
            (Number::S32(a), Number::S32(b)) => Number::S32(a.wrapping_mul(b)),
            (Number::S64(a), Number::S64(b)) => Number::S64(a.wrapping_mul(b)),
            (Number::F32(a), Number::F32(b)) => Number::F32(a * b),
            (Number::F64(a), Number::F64(b)) => Number::F64(a * b),
            (a, b) => panic!("Type mismatch: {:?} * {:?}", a, b),
        }
    }
}

impl BitOr for Number {
    type Output = Self;

    fn bitor(self, other: Self) -> Self {
        match (self, other) {
            (Number::I32(a), Number::I32(b)) => Number::I32(a | b),
            (Number::I64(a), Number::I64(b)) => Number::I64(a | b),
            (Number::U32(a), Number::U32(b)) => Number::U32(a | b),
            (Number::U64(a), Number::U64(b)) => Number::U64(a | b),
            (Number::S32(a), Number::S32(b)) => Number::S32(a | b),
            (Number::S64(a), Number::S64(b)) => Number::S64(a | b),
            n @ (Number::F32(_), Number::F32(_)) => panic!("Invalid type: {:?} | {:?}", n.0, n.1),
            n @ (Number::F64(_), Number::F64(_)) => panic!("Invalid type: {:?} | {:?}", n.0, n.1),
            (a, b) => panic!("Type mismatch: {:?} | {:?}", a, b),
        }
    }
}

impl BitAnd for Number {
    type Output = Self;

    fn bitand(self, other: Self) -> Self {
        match (self, other) {
            (Number::I32(a), Number::I32(b)) => Number::I32(a & b),
            (Number::I64(a), Number::I64(b)) => Number::I64(a & b),
            (Number::U32(a), Number::U32(b)) => Number::U32(a & b),
            (Number::U64(a), Number::U64(b)) => Number::U64(a & b),
            (Number::S32(a), Number::S32(b)) => Number::S32(a & b),
            (Number::S64(a), Number::S64(b)) => Number::S64(a & b),
            n @ (Number::F32(_), Number::F32(_)) => panic!("Invalid type: {:?} & {:?}", n.0, n.1),
            n @ (Number::F64(_), Number::F64(_)) => panic!("Invalid type: {:?} & {:?}", n.0, n.1),
            (a, b) => panic!("Type mismatch: {:?} & {:?}", a, b),
        }
    }
}

impl BitXor for Number {
    type Output = Self;

    fn bitxor(self, other: Self) -> Self {
        match (self, other) {
            (Number::I32(a), Number::I32(b)) => Number::I32(a ^ b),
            (Number::I64(a), Number::I64(b)) => Number::I64(a ^ b),
            (Number::U32(a), Number::U32(b)) => Number::U32(a ^ b),
            (Number::U64(a), Number::U64(b)) => Number::U64(a ^ b),
            (Number::S32(a), Number::S32(b)) => Number::S32(a ^ b),
            (Number::S64(a), Number::S64(b)) => Number::S64(a ^ b),
            n @ (Number::F32(_), Number::F32(_)) => panic!("Invalid type: {:?} ^ {:?}", n.0, n.1),
            n @ (Number::F64(_), Number::F64(_)) => panic!("Invalid type: {:?} ^ {:?}", n.0, n.1),
            (a, b) => panic!("Type mismatch: {:?} ^ {:?}", a, b),
        }
    }
}

impl Div for Number {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        match (self, other) {
            (Number::I32(a), Number::I32(b)) => Number::I32(a / b),
            (Number::I64(a), Number::I64(b)) => Number::I64(a / b),
            (Number::U32(a), Number::U32(b)) => Number::U32(a / b),
            (Number::U64(a), Number::U64(b)) => Number::U64(a / b),
            (Number::S32(a), Number::S32(b)) => Number::S32(a / b),
            (Number::S64(a), Number::S64(b)) => Number::S64(a / b),
            (Number::F32(a), Number::F32(b)) => Number::F32(a / b),
            (Number::F64(a), Number::F64(b)) => Number::F64(a / b),
            (a, b) => panic!("Type mismatch: {:?} / {:?}", a, b),
        }
    }
}

impl Shl for Number {
    type Output = Self;

    fn shl(self, other: Self) -> Self {
        match (self, other) {
            (Number::I32(a), Number::I32(b)) => Number::I32(a.wrapping_shl(b)),
            (Number::I64(a), Number::I64(b)) => Number::I64(a.wrapping_shl(b as u32)),
            (Number::U32(a), Number::U32(b)) => Number::U32(a.wrapping_shl(b)),
            (Number::U64(a), Number::U64(b)) => Number::U64(a.wrapping_shl(b as u32)),
            (Number::S32(a), Number::S32(b)) => Number::S32(a.wrapping_shl(b as u32)),
            (Number::S64(a), Number::S64(b)) => Number::S64(a.wrapping_shl(b as u32)),
            (a, b) => panic!("Type mismatch: {:?} << {:?}", a, b),
        }
    }
}

impl Shr for Number {
    type Output = Self;

    fn shr(self, other: Self) -> Self {
        match (self, other) {
            (Number::I32(a), Number::I32(b)) => Number::I32(a.wrapping_shr(b)),
            (Number::I64(a), Number::I64(b)) => Number::I64(a.wrapping_shr(b as u32)),
            (Number::U32(a), Number::U32(b)) => Number::U32(a.wrapping_shr(b)),
            (Number::U64(a), Number::U64(b)) => Number::U64(a.wrapping_shr(b as u32)),
            (Number::S32(a), Number::S32(b)) => Number::S32(a.wrapping_shr(b as u32)),
            (Number::S64(a), Number::S64(b)) => Number::S64(a.wrapping_shr(b as u32)),
            // TODO: these could be an unreachable for production code (idk if there's a perf diff between panic and unreachable, but there should be, right?)
            n @ (Number::F32(_), Number::F32(_)) => panic!("Invalid type: {:?} >> {:?}", n.0, n.1),
            n @ (Number::F64(_), Number::F64(_)) => panic!("Invalid type: {:?} >> {:?}", n.0, n.1),
            (a, b) => panic!("Type mismatch: {:?} >> {:?}", a, b),
        }
    }
}

impl Rem for Number {
    type Output = Self;

    fn rem(self, other: Self) -> Self {
        match (self, other) {
            (Number::I32(a), Number::I32(b)) => Number::I32(a % b),
            (Number::I64(a), Number::I64(b)) => Number::I64(a % b),
            (Number::U32(a), Number::U32(b)) => Number::U32(a % b),
            (Number::U64(a), Number::U64(b)) => Number::U64(a % b),
            (Number::S32(a), Number::S32(b)) => Number::S32(a.wrapping_rem(b)),
            (Number::S64(a), Number::S64(b)) => Number::S64(a.wrapping_rem(b)),
            n @ (Number::F32(_), Number::F32(_)) => panic!("Invalid type: {:?} % {:?}", n.0, n.1),
            n @ (Number::F64(_), Number::F64(_)) => panic!("Invalid type: {:?} % {:?}", n.0, n.1),
            (a, b) => panic!("Type mismatch: {:?} % {:?}", a, b),
        }
    }
}

impl Neg for Number {
    type Output = Self;

    fn neg(self) -> Self {
        match self {
            Number::S32(n) => Number::S32(-n),
            Number::S64(n) => Number::S64(-n),
            Number::F32(n) => Number::F32(-n),
            Number::F64(n) => Number::F64(-n),
            n => panic!("Invalid type: -{:?}", n),
        }
    }
}
