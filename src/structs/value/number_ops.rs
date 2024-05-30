use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Neg, Rem, Shl, Shr, Sub};

use super::Number;

impl Add for Number {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        match (self, other) {
            (Number::U32(a), Number::U32(b)) => Number::U32(a + b),
            (Number::U64(a), Number::U64(b)) => Number::U64(a + b),
            (Number::S32(a), Number::S32(b)) => Number::S32(a + b),
            (Number::S64(a), Number::S64(b)) => Number::S64(a + b),
            (Number::F32(a), Number::F32(b)) => Number::F32(a + b),
            (Number::F64(a), Number::F64(b)) => Number::F64(a + b),
            _ => panic!("Type mismatch"),
        }
    }
}

impl Sub for Number {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        match (self, other) {
            (Number::U32(a), Number::U32(b)) => Number::U32(a - b),
            (Number::U64(a), Number::U64(b)) => Number::U64(a - b),
            (Number::S32(a), Number::S32(b)) => Number::S32(a - b),
            (Number::S64(a), Number::S64(b)) => Number::S64(a - b),
            (Number::F32(a), Number::F32(b)) => Number::F32(a - b),
            (Number::F64(a), Number::F64(b)) => Number::F64(a - b),
            _ => panic!("Type mismatch"),
        }
    }
}

impl Mul for Number {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        match (self, other) {
            (Number::U32(a), Number::U32(b)) => Number::U32(a * b),
            (Number::U64(a), Number::U64(b)) => Number::U64(a * b),
            (Number::S32(a), Number::S32(b)) => Number::S32(a * b),
            (Number::S64(a), Number::S64(b)) => Number::S64(a * b),
            (Number::F32(a), Number::F32(b)) => Number::F32(a * b),
            (Number::F64(a), Number::F64(b)) => Number::F64(a * b),
            _ => panic!("Type mismatch"),
        }
    }
}

impl BitOr for Number {
    type Output = Self;

    fn bitor(self, other: Self) -> Self {
        match (self, other) {
            (Number::U32(a), Number::U32(b)) => Number::U32(a | b),
            (Number::U64(a), Number::U64(b)) => Number::U64(a | b),
            (Number::S32(a), Number::S32(b)) => Number::S32(a | b),
            (Number::S64(a), Number::S64(b)) => Number::S64(a | b),
            _ => panic!("Type mismatch"),
        }
    }
}

impl BitAnd for Number {
    type Output = Self;

    fn bitand(self, other: Self) -> Self {
        match (self, other) {
            (Number::U32(a), Number::U32(b)) => Number::U32(a & b),
            (Number::U64(a), Number::U64(b)) => Number::U64(a & b),
            (Number::S32(a), Number::S32(b)) => Number::S32(a & b),
            (Number::S64(a), Number::S64(b)) => Number::S64(a & b),
            _ => panic!("Type mismatch"),
        }
    }
}

impl BitXor for Number {
    type Output = Self;

    fn bitxor(self, other: Self) -> Self {
        match (self, other) {
            (Number::U32(a), Number::U32(b)) => Number::U32(a ^ b),
            (Number::U64(a), Number::U64(b)) => Number::U64(a ^ b),
            (Number::S32(a), Number::S32(b)) => Number::S32(a ^ b),
            (Number::S64(a), Number::S64(b)) => Number::S64(a ^ b),
            _ => panic!("Type mismatch"),
        }
    }
}

impl Div for Number {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        match (self, other) {
            (Number::U32(a), Number::U32(b)) => Number::U32(a / b),
            (Number::U64(a), Number::U64(b)) => Number::U64(a / b),
            (Number::S32(a), Number::S32(b)) => Number::S32(a / b),
            (Number::S64(a), Number::S64(b)) => Number::S64(a / b),
            (Number::F32(a), Number::F32(b)) => Number::F32(a / b),
            (Number::F64(a), Number::F64(b)) => Number::F64(a / b),
            _ => panic!("Type mismatch"),
        }
    }
}

impl Shl for Number {
    type Output = Self;

    fn shl(self, other: Self) -> Self {
        match (self, other) {
            (Number::U32(a), Number::U32(b)) => Number::U32(a << b),
            (Number::U64(a), Number::U64(b)) => Number::U64(a << b),
            (Number::S32(a), Number::S32(b)) => Number::S32(a << b),
            (Number::S64(a), Number::S64(b)) => Number::S64(a << b),
            _ => panic!("Type mismatch"),
        }
    }
}

impl Shr for Number {
    type Output = Self;

    fn shr(self, other: Self) -> Self {
        match (self, other) {
            (Number::U32(a), Number::U32(b)) => Number::U32(a >> b),
            (Number::U64(a), Number::U64(b)) => Number::U64(a >> b),
            (Number::S32(a), Number::S32(b)) => Number::S32(a >> b),
            (Number::S64(a), Number::S64(b)) => Number::S64(a >> b),
            // TODO: this could be an unreachable for production code (idk if there's a perf diff between panic and unreachable, but there should be, right?)
            _ => panic!("Type mismatch"),
        }
    }
}

impl Rem for Number {
    type Output = Self;

    fn rem(self, other: Self) -> Self {
        match (self, other) {
            (Number::U32(a), Number::U32(b)) => Number::U32(a % b),
            (Number::U64(a), Number::U64(b)) => Number::U64(a % b),
            (Number::S32(a), Number::S32(b)) => Number::S32(a % b),
            (Number::S64(a), Number::S64(b)) => Number::S64(a % b),
            _ => panic!("Type mismatch"),
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
            _ => panic!("Type mismatch"),
        }
    }
}
