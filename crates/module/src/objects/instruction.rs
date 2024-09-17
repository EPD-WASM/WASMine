use rkyv::{Archive, Deserialize, Serialize};
use wasm_types::{BlockType, FuncIdx, LabelIdx, TableIdx, TypeIdx};

pub enum Instructon {}

#[derive(Debug, Clone, Default, Archive, Deserialize, Serialize)]
pub enum ControlInstruction {
    Nop,
    #[default]
    Unreachable,
    Block(BlockType),
    Loop(BlockType),
    IfElse(BlockType),
    Br(LabelIdx),
    BrIf(LabelIdx),
    BrTable(LabelIdx, Vec<LabelIdx>),
    Return,
    Call(FuncIdx),
    CallIndirect(TypeIdx, TableIdx),

    // these are not real wasm terminators, but rather a signal to our parser that we reached the end of a block / the else statement
    End,
    Else,
}
