#[derive(Debug, Clone)]
pub(crate) enum InstructionType {
    Numeric(NumericInstructionCategory),
    Reference(ReferenceInstructionType),
    Vector(VectorInstructionCategory),
    Parametric(ParametricInstructionType),
    Variable(VariableInstructionType),
    Table(TableInstructionCategory),
    Memory(MemoryInstructionCategory),
    Control(ControlInstructionType),
}

#[derive(Debug, Clone)]
pub(crate) enum NumericInstructionCategory {
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

#[derive(Debug, Clone)]
pub(crate) enum IUnaryOp {
    Clz,
    Ctz,
    Popcnt,
}

#[derive(Debug, Clone)]
pub(crate) enum FUnaryOp {
    Abs,
    Neg,
    Sqrt,
    Ceil,
    Floor,
    Trunc,
    Nearest,
}

#[derive(Debug, Clone)]
pub(crate) enum IBinaryOp {
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

#[derive(Debug, Clone)]
pub(crate) enum FBinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Min,
    Max,
    Copysign,
}

#[derive(Debug, Clone)]
pub(crate) enum ITestOp {
    Eqz,
}

#[derive(Debug, Clone)]
pub(crate) enum IRelationalOp {
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

#[derive(Debug, Clone)]
pub(crate) enum FRelationalOp {
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
}

#[derive(Debug, Clone)]
pub(crate) enum ConversionOp {
    Wrap,
    Extend,
    Trunc,
    Demote,
    Promote,
    Convert,
    Reinterpret,
}

#[derive(Debug, Clone)]
pub(crate) enum VectorInstructionCategory {
    // TODO: add vector instructions
}

#[derive(Debug, Clone)]
pub(crate) enum ReferenceInstruction {
    RefNull,
    RefIsNull,
    RefFunc,
}

#[derive(Debug, Clone)]
pub(crate) enum ParametricInstructionType {
    Drop,
    Select,
}

#[derive(Debug, Clone)]
pub(crate) enum VariableInstructionType {
    LocalGet,
    LocalSet,
    LocalTee,
    GlobalGet,
    GlobalSet,
}

#[derive(Debug, Clone)]
pub(crate) enum TableInstructionCategory {
    Get,
    Set,
    Size,
    Grow,
    Fill,
    Copy,
    Init,
    Drop,
}

#[derive(Debug, Clone)]
pub(crate) enum MemoryInstructionCategory {
    Load(LoadOp),
    Store(StoreOp),
    Memory(MemoryOp),
}

#[derive(Debug, Clone)]
pub(crate) enum LoadOp {
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

#[derive(Debug, Clone)]
pub(crate) enum StoreOp {
    INNStore,
    FNNStore,
    INNStore8,
    INNStore16,
    INNStore32,
    // TODO: add vector instructions
}

#[derive(Debug, Clone)]
pub(crate) enum MemoryOp {
    Size,
    Grow,
    Fill,
    Copy,
    Init,
    Drop,
}

#[derive(Debug, Clone)]
pub(crate) enum ControlInstructionType {
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

#[derive(Debug, Clone)]
pub(crate) enum ReferenceInstructionType {
    RefNull,
    RefIsNull,
    RefFunc,
}
