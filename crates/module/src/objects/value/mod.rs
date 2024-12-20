// https://webassembly.github.io/spec/core/exec/runti`me`.html

use crate::utils::numeric_transmutes::{Bit32, Bit64};
use rkyv::{Archive, Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use wasm_types::{FuncIdx, GlobalIdx, NumType, RefType, ValType};

mod number_impls;
mod number_ops;

#[derive(Debug, Clone, PartialEq, PartialOrd, Archive, Deserialize, Serialize)]
pub enum Number {
    I32(u32),
    I64(u64),
    U32(u32),
    U64(u64),
    S32(i32),
    S64(i64),
    F32(f32),
    F64(f64),
}

impl Default for Number {
    fn default() -> Self {
        Number::I32(0)
    }
}

impl Display for Number {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Number::I32(n) => write!(f, "{n}"),
            Number::I64(n) => write!(f, "{n}"),
            Number::U32(n) => write!(f, "{n}"),
            Number::U64(n) => write!(f, "{n}"),
            Number::S32(n) => write!(f, "{n}"),
            Number::S64(n) => write!(f, "{n}"),
            Number::F32(n) => write!(f, "{n}"),
            Number::F64(n) => write!(f, "{n}"),
        }
    }
}

pub type Vector = [u8; 16];

pub type FunctionReference = FuncIdx;
pub type ExternReference = u32;

#[derive(Debug, Clone, PartialEq, Archive, Deserialize, Serialize)]
pub enum Reference {
    Null,
    Function(FuncIdx),
    Extern(u64),
}

impl Reference {
    pub fn as_u64(&self) -> u64 {
        match *self {
            Reference::Null => 0,
            Reference::Function(f) => f as u64,
            Reference::Extern(e) => e as u64,
        }
    }

    pub fn from_u32(n: u32, t: &RefType) -> Self {
        if n == 0 {
            return Reference::Null;
        }
        match t {
            RefType::FunctionReference => Reference::Function(n as FunctionReference),
            RefType::ExternReference => panic!("Cannot create pointer from u32"),
        }
    }
}

impl Display for Reference {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Reference::Null => write!(f, "null"),
            Reference::Function(idx) => write!(f, "func[{}]", *idx as u64),
            Reference::Extern(idx) => write!(f, "extern[{}]", *idx),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Archive, Deserialize, Serialize)]
pub enum Value {
    Number(Number),
    Vector(Vector),
    Reference(Reference),
}

impl Value {
    pub fn trans_to_u64(&self) -> u64 {
        match self {
            Value::Number(n) => n.trans_to_u64(),
            Value::Vector(_) => todo!("Vector to u64 lmao"),
            Value::Reference(r) => r.as_u64(),
        }
    }

    pub fn from_u64(n: u64, t: ValType) -> Self {
        match t {
            ValType::Number(t) => Value::Number(Number::trans_from_u64(n, &t)),
            ValType::Reference(t) => Value::Reference(Reference::from_u32(n as u32, &t)),
            ValType::VecType => todo!(),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Value::Number(n) => write!(f, "{}", *n),
            Value::Vector(v) => write!(f, "{v:?}"),
            Value::Reference(r) => write!(f, "{}", *r),
        }
    }
}

impl Value {
    pub fn r#type(&self) -> ValType {
        match self {
            Value::Number(Number::I32(_))
            | Value::Number(Number::U32(_))
            | Value::Number(Number::S32(_)) => ValType::Number(NumType::I32),
            Value::Number(Number::I64(_))
            | Value::Number(Number::U64(_))
            | Value::Number(Number::S64(_)) => ValType::Number(NumType::I64),
            Value::Number(Number::F32(_)) => ValType::Number(NumType::F32),
            Value::Number(Number::F64(_)) => ValType::Number(NumType::F64),
            Value::Vector(_) => ValType::VecType,
            Value::Reference(Reference::Function(_)) => {
                ValType::Reference(RefType::FunctionReference)
            }
            Value::Reference(Reference::Extern(_)) => ValType::Reference(RefType::ExternReference),
            Value::Reference(Reference::Null) => ValType::Reference(RefType::FunctionReference),
        }
    }

    #[inline]
    pub const fn i32(value: u32) -> Self {
        Value::Number(Number::I32(value))
    }

    #[inline]
    pub const fn i64(value: u64) -> Self {
        Value::Number(Number::I64(value))
    }

    #[inline]
    pub const fn f32(value: f32) -> Self {
        Value::Number(Number::F32(value))
    }

    #[inline]
    pub const fn f64(value: f64) -> Self {
        Value::Number(Number::F64(value))
    }

    #[inline]
    pub const fn funcref(value: FuncIdx) -> Self {
        Value::Reference(Reference::Function(value))
    }

    #[inline]
    pub const fn externref(value: u64) -> Self {
        Value::Reference(Reference::Extern(value))
    }

    #[inline]
    pub const fn v128(value: [u8; 16]) -> Self {
        Value::Vector(value)
    }
}

/// Like Value, but without the tag to decrease size and ffi compatible
#[repr(C)]
#[derive(Clone, Copy)]
pub union ValueRaw {
    i32: u32,
    i64: u64,

    /// u32 instead of f32 to avoid signalling to non signalling NaN conversion
    f32: u32,
    /// u64 instead of f64 to avoid signalling to non signalling NaN conversion
    f64: u64,

    v128: [u8; 16],
    funcref: FuncIdx,
    externref: u64,
}

const ZEROED: ValueRaw = ValueRaw { v128: [0; 16] };

impl ValueRaw {
    #[inline]
    pub fn i32(value: i32) -> Self {
        // zeroing. According to godbolt this generates two additional MOVs
        let mut ret = ZEROED;
        ret.i32 = value.trans_u32();
        ret
    }

    #[inline]
    pub fn i64(value: i64) -> Self {
        let mut ret = ZEROED;
        ret.i64 = value.trans_u64();
        ret
    }

    #[inline]
    pub const fn u32(value: u32) -> Self {
        let mut ret = ZEROED;
        ret.i32 = value;
        ret
    }

    #[inline]
    pub const fn u64(value: u64) -> Self {
        let mut ret = ZEROED;
        ret.i64 = value;
        ret
    }

    #[inline]
    pub const fn f32(value: u32) -> Self {
        let mut ret = ZEROED;
        ret.f32 = value;
        ret
    }

    #[inline]
    pub const fn f64(value: u64) -> Self {
        let mut ret = ZEROED;
        ret.f64 = value;
        ret
    }

    #[inline]
    pub const fn v128(value: [u8; 16]) -> Self {
        ValueRaw { v128: value }
    }

    #[inline]
    pub const fn funcref(value: FuncIdx) -> Self {
        let mut ret = ZEROED;
        ret.funcref = value;
        ret
    }

    #[inline]
    pub const fn externref(value: u64) -> Self {
        let mut ret = ZEROED;
        ret.externref = value;
        ret
    }

    #[inline]
    pub fn as_i32(self) -> i32 {
        unsafe { self.i32 }.trans_i32()
    }

    #[inline]
    pub fn as_i64(self) -> i64 {
        unsafe { self.i64 }.trans_i64()
    }

    #[inline]
    pub const fn as_u32(self) -> u32 {
        unsafe { self.i32 }
    }

    #[inline]
    pub const fn as_u64(self) -> u64 {
        unsafe { self.i64 }
    }

    #[inline]
    pub const fn as_f32(self) -> u32 {
        unsafe { self.f32 }
    }

    #[inline]
    pub const fn as_f64(self) -> u64 {
        unsafe { self.f64 }
    }

    #[inline]
    pub const fn as_v128(self) -> [u8; 16] {
        unsafe { self.v128 }
    }

    #[inline]
    pub const fn as_funcref(self) -> FuncIdx {
        unsafe { self.funcref }
    }

    #[inline]
    pub const fn as_externref(self) -> u64 {
        unsafe { self.externref }
    }
}

impl PartialEq for ValueRaw {
    fn eq(&self, other: &Self) -> bool {
        self.as_v128() == other.as_v128()
    }
}

impl From<Value> for ValueRaw {
    fn from(val: Value) -> ValueRaw {
        match val {
            Value::Number(Number::I32(val)) => ValueRaw::u32(val),
            Value::Number(Number::S32(val)) => ValueRaw::i32(val),
            Value::Number(Number::U32(val)) => ValueRaw::u32(val),
            Value::Number(Number::I64(val)) => ValueRaw::u64(val),
            Value::Number(Number::S64(val)) => ValueRaw::i64(val),
            Value::Number(Number::U64(val)) => ValueRaw::u64(val),
            Value::Number(Number::F32(val)) => ValueRaw::f32(val.to_bits()),
            Value::Number(Number::F64(val)) => ValueRaw::f64(val.to_bits()),
            Value::Vector(val) => ValueRaw::v128(val),
            Value::Reference(Reference::Function(val)) => ValueRaw::funcref(val),
            Value::Reference(Reference::Extern(val)) => ValueRaw::externref(val),
            Value::Reference(Reference::Null) => ValueRaw::u64(u64::MAX),
        }
    }
}

impl std::fmt::Debug for ValueRaw {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.as_v128())
    }
}

impl From<i32> for ValueRaw {
    fn from(val: i32) -> ValueRaw {
        ValueRaw::i32(val)
    }
}

impl From<i64> for ValueRaw {
    fn from(val: i64) -> ValueRaw {
        ValueRaw::i64(val)
    }
}

impl From<u32> for ValueRaw {
    fn from(val: u32) -> ValueRaw {
        ValueRaw::u32(val)
    }
}

impl From<u64> for ValueRaw {
    fn from(val: u64) -> ValueRaw {
        ValueRaw::u64(val)
    }
}

impl From<f32> for ValueRaw {
    fn from(val: f32) -> ValueRaw {
        ValueRaw::f32(val.to_bits())
    }
}

impl From<f64> for ValueRaw {
    fn from(val: f64) -> ValueRaw {
        ValueRaw::f64(val.to_bits())
    }
}

impl From<[u8; 16]> for ValueRaw {
    fn from(val: [u8; 16]) -> ValueRaw {
        ValueRaw::v128(val)
    }
}

impl From<ValueRaw> for i32 {
    fn from(val: ValueRaw) -> i32 {
        val.as_i32()
    }
}

impl From<ValueRaw> for i64 {
    fn from(val: ValueRaw) -> i64 {
        val.as_i64()
    }
}

impl From<ValueRaw> for u32 {
    fn from(val: ValueRaw) -> u32 {
        val.as_u32()
    }
}

impl From<ValueRaw> for u64 {
    fn from(val: ValueRaw) -> u64 {
        val.as_u64()
    }
}

impl From<ValueRaw> for f32 {
    fn from(val: ValueRaw) -> f32 {
        f32::from_bits(val.as_f32())
    }
}

impl From<ValueRaw> for f64 {
    fn from(val: ValueRaw) -> f64 {
        f64::from_bits(val.as_f64())
    }
}

impl From<ValueRaw> for [u8; 16] {
    fn from(val: ValueRaw) -> [u8; 16] {
        val.as_v128()
    }
}

impl Value {
    pub fn from_raw(raw: ValueRaw, val_ty: ValType) -> Self {
        match val_ty {
            ValType::Number(NumType::I32) => Value::i32(raw.as_u32()),
            ValType::Number(NumType::I64) => Value::i64(raw.as_u64()),
            ValType::Number(NumType::F32) => Value::f32(f32::from_bits(raw.as_f32())),
            ValType::Number(NumType::F64) => Value::f64(f64::from_bits(raw.as_f64())),
            ValType::Reference(RefType::ExternReference) => {
                if raw.as_externref()
                    == ValueRaw::from(Value::Reference(Reference::Null)).as_externref()
                {
                    Value::Reference(Reference::Null)
                } else {
                    Value::externref(raw.as_externref())
                }
            }
            ValType::Reference(RefType::FunctionReference) => {
                if raw.as_funcref()
                    == ValueRaw::from(Value::Reference(Reference::Null)).as_funcref()
                {
                    Value::Reference(Reference::Null)
                } else {
                    Value::funcref(raw.as_funcref())
                }
            }
            ValType::VecType => Value::v128(raw.as_v128()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Archive, Deserialize, Serialize)]
pub enum ConstantValue {
    V(Value),
    // we can't resolve the value of imported globals at parsing time
    Global(GlobalIdx),
    // we can't resolve the function pointer at parsing time
    FuncPtr(FuncIdx),
}
