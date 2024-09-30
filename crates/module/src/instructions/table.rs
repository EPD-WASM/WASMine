use super::*;
use wasm_types::*;

#[derive(Debug, Clone)]
pub struct TableSetInstruction {
    pub table_idx: TableIdx,
    pub in1: VariableID,
    pub idx: VariableID,
    pub input_type: ValType,
}

impl Instruction for TableSetInstruction {
    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let table_idx = i.read_immediate()?;
        let input_type = i.read_value_type()?;
        let in1: VariableID = i.read_variable()?;
        let idx: VariableID = i.read_variable()?;
        Ok(TableSetInstruction {
            table_idx,
            in1,
            idx,
            input_type,
        })
    }
}

impl Display for TableSetInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "table.set(i32 {}, i32 {}) %{}",
            self.table_idx, self.input_type, self.in1
        )
    }
}

#[derive(Debug, Clone)]
pub struct TableGetInstruction {
    pub table_idx: TableIdx,
    pub idx: VariableID,
    pub out1: VariableID,
}

impl Instruction for TableGetInstruction {
    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let table_idx = i.read_immediate()?;
        let idx = i.read_variable()?;
        let out1 = i.read_variable()?;
        Ok(TableGetInstruction {
            table_idx,
            idx,
            out1,
        })
    }
}

impl Display for TableGetInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "%{} = table.get(i32 {})", self.table_idx, self.out1)
    }
}

#[derive(Debug, Clone)]
pub struct TableGrowInstruction {
    pub table_idx: TableIdx,
    pub size: VariableID,
    pub value_to_fill: VariableID,
    pub out1: VariableID,
}

impl Instruction for TableGrowInstruction {
    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let table_idx = i.read_immediate()?;
        let size = i.read_variable()?;
        let value_to_fill = i.read_variable()?;
        let out1 = i.read_variable()?;
        Ok(TableGrowInstruction {
            table_idx,
            size,
            value_to_fill,
            out1,
        })
    }
}

impl Display for TableGrowInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "%{} = table.grow(i32 {}) %{}, %{}",
            self.out1, self.table_idx, self.size, self.value_to_fill
        )
    }
}

#[derive(Debug, Clone)]
pub struct TableSizeInstruction {
    pub table_idx: TableIdx,
    pub out1: VariableID,
}

impl Instruction for TableSizeInstruction {
    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let table_idx = i.read_immediate()?;
        let out1 = i.read_variable()?;
        Ok(TableSizeInstruction { table_idx, out1 })
    }
}

impl Display for TableSizeInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "%{}: i32 = table.size(i32 {})",
            self.out1, self.table_idx
        )
    }
}

#[derive(Debug, Clone)]
pub struct TableFillInstruction {
    pub table_idx: TableIdx,
    pub i: VariableID,
    pub n: VariableID,
    pub ref_value: VariableID,
}

impl Instruction for TableFillInstruction {
    fn deserialize(
        in_: &mut InstructionDecoder,
        _: InstructionType,
    ) -> Result<Self, DecodingError> {
        let table_idx = in_.read_immediate()?;
        let i = in_.read_variable()?;
        let n = in_.read_variable()?;
        let ref_value = in_.read_variable()?;
        Ok(TableFillInstruction {
            table_idx,
            i,
            n,
            ref_value,
        })
    }
}

impl Display for TableFillInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "table.fill(i32 {}) %{}, %{}, %{}",
            self.table_idx, self.i, self.n, self.ref_value
        )
    }
}

#[derive(Debug, Clone)]
pub struct TableCopyInstruction {
    pub table_idx_x: TableIdx,
    pub table_idx_y: TableIdx,
    pub n: VariableID,
    pub s: VariableID,
    pub d: VariableID,
}

impl Instruction for TableCopyInstruction {
    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let table_idx_x = i.read_immediate()?;
        let table_idx_y = i.read_immediate()?;
        let n = i.read_variable()?;
        let s = i.read_variable()?;
        let d = i.read_variable()?;
        Ok(TableCopyInstruction {
            table_idx_x,
            table_idx_y,
            n,
            s,
            d,
        })
    }
}

impl Display for TableCopyInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "table.copy(i32 {}, i32 {}) %{}, %{}, %{}",
            self.table_idx_x, self.table_idx_y, self.n, self.s, self.d
        )
    }
}

#[derive(Debug, Clone)]
pub struct TableInitInstruction {
    pub table_idx: TableIdx,
    pub elem_idx: ElemIdx,
    pub n: VariableID,
    pub s: VariableID,
    pub d: VariableID,
}

impl Instruction for TableInitInstruction {
    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let table_idx = i.read_immediate()?;
        let elem_idx = i.read_immediate()?;
        let n = i.read_variable()?;
        let s = i.read_variable()?;
        let d = i.read_variable()?;
        Ok(TableInitInstruction {
            table_idx,
            elem_idx,
            n,
            s,
            d,
        })
    }
}

impl Display for TableInitInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "table.init(i32 {}, i32 {}) %{}, %{}, %{}",
            self.table_idx, self.elem_idx, self.n, self.s, self.d
        )
    }
}

#[derive(Debug, Clone)]
pub struct ElemDropInstruction {
    pub elem_idx: ElemIdx,
}

impl Instruction for ElemDropInstruction {
    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let elem_idx = i.read_immediate()?;
        Ok(ElemDropInstruction { elem_idx })
    }
}

impl Display for ElemDropInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "elem.drop(i32 {})", self.elem_idx)
    }
}
