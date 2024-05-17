/// https://webassembly.github.io/spec/core/syntax/types.html#number-types
#[derive(Debug, Clone, PartialEq, Copy)]
pub(crate) enum NumType {
    I32,
    I64,
    F32,
    F64,
}

/// https://webassembly.github.io/spec/core/syntax/types.html#reference-types
#[derive(Debug, Clone, PartialEq, Copy, Default)]
pub(crate) enum RefType {
    #[default]
    FunctionReference,
    ExternReference,
}

/// https://webassembly.github.io/spec/core/syntax/types.html#value-types
#[derive(Debug, Clone, PartialEq, Default, Copy)]
pub(crate) enum ValType {
    Number(NumType),
    Reference(RefType),
    /// https://webassembly.github.io/spec/core/syntax/types.html#vector-types
    #[default]
    VecType,
}

/// https://webassembly.github.io/spec/core/syntax/types.html#result-types
pub(crate) type ResType = Vec<ValType>;

// https://webassembly.github.io/spec/core/syntax/types.html#result-types
pub(crate) type FuncType = (ResType, ResType);

#[derive(Debug, Clone, Copy)]
pub(crate) struct LimType {
    pub(crate) min: u32,
    pub(crate) max: Option<u32>,
}

pub(crate) type MemType = LimType;

pub(crate) type TypeIdx = u32;
pub(crate) type FuncIdx = u32;
pub(crate) type TableIdx = u32;
pub(crate) type MemIdx = u32;
pub(crate) type GlobalIdx = u32;
pub(crate) type ElemIdx = u32;
pub(crate) type DataIdx = u32;
pub(crate) type LocalIdx = u32;
pub(crate) type LabelIdx = u32;

#[derive(Debug, Clone, Copy)]
pub(crate) struct TableType {
    pub(crate) ref_type: RefType,
    pub(crate) lim: LimType,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum GlobalType {
    Mut(ValType),
    Const(ValType),
}

#[derive(Debug, Clone)]
pub(crate) enum ExternType {
    Func(FuncType),
    Table(TableType),
    Mem(MemType),
    Global(GlobalType),
}

pub(crate) type Name = String;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Section {
    Custom,
    Type,
    Import,
    Function,
    Table,
    Memory,
    Global,
    Export,
    Start,
    Element,
    Code,
    Data,
    DataCount,
}

// https://webassembly.github.io/spec/core/bikeshed/#syntax-blocktype
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) enum BlockType {
    FunctionSig(TypeIdx),
    ShorthandFunc(ValType),
    #[default]
    Empty,
}
