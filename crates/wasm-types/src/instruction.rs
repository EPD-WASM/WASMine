#![allow(unreachable_code)] // rustc otherwise reports unreachable code for the InstructionType::Vector variant

use crate::*;

#[derive(Debug, Clone, PartialEq, Archive, Deserialize, Serialize)]
pub enum InstructionType {
    Numeric(NumericInstructionCategory),
    Reference(ReferenceInstructionType),
    Vector,
    Parametric(ParametricInstructionType),
    Variable(VariableInstructionType),
    Table(TableInstructionCategory),
    Memory(MemoryInstructionCategory),
    Control(ControlInstructionType),
    Meta(MetaInstructionType),
}

#[derive(Debug, Clone, PartialEq, Archive, Deserialize, Serialize)]
pub enum NumericInstructionCategory {
    /// Constants: return a static constant.
    Constant,
    /// Unary Operators: consume one operand and produce one result of the respective type.
    IUnary(IUnaryOp),
    /// Binary Operators: consume two operands and produce one result of the respective type.
    IBinary(IBinaryOp),
    /// Tests: consume one operand of the respective type and produce a Boolean integer result.
    ITest(ITestOp),
    /// Comparisons: consume two operands of the respective type and produce a Boolean integer result.
    IRelational(IRelationalOp),
    /// Conversions: consume a value of one type and produce a result of another.
    Conversion(ConversionOp),

    FUnary(FUnaryOp),
    FBinary(FBinaryOp),
    FRelational(FRelationalOp),
}

#[derive(Debug, Clone, PartialEq, Archive, Deserialize, Serialize)]
pub enum IUnaryOp {
    Clz,
    Ctz,
    Popcnt,
}

#[derive(Debug, Clone, PartialEq, Archive, Deserialize, Serialize)]
pub enum FUnaryOp {
    Abs,
    Neg,
    Sqrt,
    Ceil,
    Floor,
    Trunc,
    Nearest,
}

#[derive(Debug, Clone, PartialEq, Archive, Deserialize, Serialize)]
pub enum IBinaryOp {
    Add,
    Sub,
    Mul,
    DivS,
    DivU,
    RemS,
    RemU,
    And,
    Or,
    Xor,
    Shl,
    ShrS,
    ShrU,
    Rotl,
    Rotr,
}

#[derive(Debug, Clone, PartialEq, Archive, Deserialize, Serialize)]
pub enum FBinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Min,
    Max,
    Copysign,
}

#[derive(Debug, Clone, PartialEq, Archive, Deserialize, Serialize)]
pub enum ITestOp {
    Eqz,
}

#[derive(Debug, Clone, PartialEq, Archive, Deserialize, Serialize)]
pub enum IRelationalOp {
    Eq,
    Ne,
    LtS,
    LtU,
    GtS,
    GtU,
    LeS,
    LeU,
    GeS,
    GeU,
}

#[derive(Debug, Clone, PartialEq, Archive, Deserialize, Serialize)]
pub enum FRelationalOp {
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
}

#[derive(Debug, Clone, PartialEq, Archive, Deserialize, Serialize)]
pub enum ConversionOp {
    Wrap,
    ExtendBits,
    ExtendType,
    Trunc,
    TruncSat,
    Demote,
    Promote,
    Convert,
    Reinterpret,
}

#[derive(Debug, Clone, PartialEq, Archive, Deserialize, Serialize)]
pub enum ParametricInstructionType {
    Drop,
    Select,
}

#[derive(Debug, Clone, PartialEq, Archive, Deserialize, Serialize)]
pub enum VariableInstructionType {
    LocalGet,
    LocalSet,
    LocalTee,
    GlobalGet,
    GlobalSet,
}

#[derive(Debug, Clone, PartialEq, Archive, Deserialize, Serialize)]
pub enum TableInstructionCategory {
    Get,
    Set,
    Size,
    Grow,
    Fill,
    Copy,
    Init,
    Drop,
}

#[derive(Debug, Clone, PartialEq, Archive, Deserialize, Serialize)]
pub enum MemoryInstructionCategory {
    Load(LoadOp),
    Store(StoreOp),
    Memory(MemoryOp),
}

#[derive(Debug, Clone, PartialEq, Archive, Deserialize, Serialize)]
pub enum LoadOp {
    INNLoad,
    FNNLoad,
    INNLoad8S,
    INNLoad8U,
    INNLoad16S,
    INNLoad16U,
    INNLoad32S,
    INNLoad32U,
    // TODO: add vector instructions
}

#[derive(Debug, Clone, PartialEq, Archive, Deserialize, Serialize)]
pub enum StoreOp {
    INNStore,
    FNNStore,
    INNStore8,
    INNStore16,
    INNStore32,
    // TODO: add vector instructions
}

#[derive(Debug, Clone, PartialEq, Archive, Deserialize, Serialize)]
pub enum MemoryOp {
    Size,
    Grow,
    Fill,
    Copy,
    Init,
    Drop,
}

#[derive(Debug, Clone, PartialEq, Archive, Deserialize, Serialize)]
pub enum ControlInstructionType {
    Nop,
    Unreachable,
    Block,
    Loop,
    IfElse,
    Br,
    BrIf,
    BrTable,
    Return,
    Call,
    CallIndirect,

    // these are not real wasm terminators, but rather a signal to our parser that we reached the end of a block / the else statement
    End,
    Else,
}

#[derive(Debug, Clone, PartialEq, Archive, Deserialize, Serialize)]
pub enum ReferenceInstructionType {
    RefNull,
    RefIsNull,
    RefFunc,
}

#[derive(Debug, Clone, PartialEq, Archive, Deserialize, Serialize)]
pub enum MetaInstructionType {
    PhiNode,
}

// https://webassembly.github.io/spec/core/bikeshed/#syntax-blocktype
#[derive(Debug, Clone, PartialEq, Default, Archive, Deserialize, Serialize)]
pub enum BlockType {
    FunctionSig(TypeIdx),
    ShorthandFunc(ValType),
    #[default]
    Empty,
}

impl Display for IUnaryOp {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            IUnaryOp::Clz => write!(f, "clz"),
            IUnaryOp::Ctz => write!(f, "ctz"),
            IUnaryOp::Popcnt => write!(f, "popcnt"),
        }
    }
}

impl Display for IBinaryOp {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                IBinaryOp::Add => "add",
                IBinaryOp::Sub => "sub",
                IBinaryOp::Mul => "mul",
                IBinaryOp::DivS => "div_s",
                IBinaryOp::DivU => "div_u",
                IBinaryOp::RemS => "rem_s",
                IBinaryOp::RemU => "rem_u",
                IBinaryOp::And => "and",
                IBinaryOp::Or => "or",
                IBinaryOp::Xor => "xor",
                IBinaryOp::Shl => "shl",
                IBinaryOp::ShrS => "shr_s",
                IBinaryOp::ShrU => "shr_u",
                IBinaryOp::Rotl => "rotl",
                IBinaryOp::Rotr => "rotr",
            }
        )
    }
}

impl Display for FUnaryOp {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                FUnaryOp::Abs => "abs",
                FUnaryOp::Neg => "neg",
                FUnaryOp::Sqrt => "sqrt",
                FUnaryOp::Ceil => "ceil",
                FUnaryOp::Floor => "floor",
                FUnaryOp::Trunc => "trunc",
                FUnaryOp::Nearest => "nearest",
            }
        )
    }
}

impl Display for FBinaryOp {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                FBinaryOp::Add => "add",
                FBinaryOp::Sub => "sub",
                FBinaryOp::Mul => "mul",
                FBinaryOp::Div => "div",
                FBinaryOp::Min => "min",
                FBinaryOp::Max => "max",
                FBinaryOp::Copysign => "copysign",
            }
        )
    }
}

impl Display for ITestOp {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "eqz")
    }
}

impl Display for IRelationalOp {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                IRelationalOp::Eq => "eq",
                IRelationalOp::Ne => "ne",
                IRelationalOp::LtS => "lt_s",
                IRelationalOp::LtU => "lt_u",
                IRelationalOp::GtS => "gt_s",
                IRelationalOp::GtU => "gt_u",
                IRelationalOp::LeS => "le_s",
                IRelationalOp::LeU => "le_u",
                IRelationalOp::GeS => "ge_s",
                IRelationalOp::GeU => "ge_u",
            }
        )
    }
}

impl Display for FRelationalOp {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                FRelationalOp::Eq => "eq",
                FRelationalOp::Ne => "ne",
                FRelationalOp::Lt => "lt",
                FRelationalOp::Gt => "gt",
                FRelationalOp::Le => "le",
                FRelationalOp::Ge => "ge",
            }
        )
    }
}

impl Display for ConversionOp {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ConversionOp::Wrap => "wrap",
                ConversionOp::ExtendBits => "extend_bits",
                ConversionOp::ExtendType => "extend_type",
                ConversionOp::Trunc => "trunc",
                ConversionOp::TruncSat => "trunc_sat",
                ConversionOp::Demote => "demote",
                ConversionOp::Promote => "promote",
                ConversionOp::Convert => "convert",
                ConversionOp::Reinterpret => "reinterpret",
            }
        )
    }
}
