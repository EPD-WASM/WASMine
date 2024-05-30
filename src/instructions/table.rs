use super::*;
use crate::parser::parsable::Parse;
use crate::structs::element::Element;
use crate::structs::table::{Table, Tablelike};
use wasm_types::*;

#[derive(Debug, Clone)]
pub(crate) struct TableSetInstruction {
    table_idx: TableIdx,
    in1: VariableID,
    input_type: ValType,
}

impl Instruction for TableSetInstruction {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.write_instruction_type(InstructionType::Table(TableInstructionCategory::Set));
        o.write_immediate(self.table_idx);
        o.write_value_type(self.input_type);
        o.write_variable(self.in1);
    }

    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let table_idx = i.read_immediate()?;
        let input_type = i.read_value_type()?;
        let in1 = i.read_variable()?;
        Ok(TableSetInstruction {
            table_idx,
            in1,
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

pub(crate) fn table_set(
    ctxt: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let table_idx = TableIdx::parse(i)?;
    let value_to_set = ctxt.pop_var();
    match value_to_set.type_ {
        ValType::Reference(_) => {}
        _ => ctxt.poison(ValidationError::Msg(
            "table value to set must be of reference type".into(),
        )),
    }
    o.write(TableSetInstruction {
        table_idx,
        in1: value_to_set.id,
        input_type: value_to_set.type_,
    });
    Ok(())
}

#[derive(Debug, Clone)]
pub(crate) struct TableGetInstruction {
    table_idx: TableIdx,
    out1: VariableID,
}

impl Instruction for TableGetInstruction {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.write_instruction_type(InstructionType::Table(TableInstructionCategory::Get));
        o.write_immediate(self.table_idx);
        o.write_variable(self.out1);
    }

    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let table_idx = i.read_immediate()?;
        let out1 = i.read_variable()?;
        Ok(TableGetInstruction { table_idx, out1 })
    }
}

impl Display for TableGetInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "%{} = table.get(i32 {})", self.table_idx, self.out1)
    }
}

pub(crate) fn table_get(
    ctxt: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let table_idx = TableIdx::parse(i)?;
    let table = match ctxt.module.tables.get(table_idx as usize) {
        Some(table) => table,
        None => {
            ctxt.poison::<()>(ValidationError::Msg(format!(
                "table with id {} not found",
                table_idx
            )));
            return Ok(());
        }
    };
    let table_ref_type = table.get_ref_type();
    let out = ctxt.create_var(ValType::Reference(table_ref_type));
    o.write(TableGetInstruction {
        table_idx,
        out1: out.id,
    });
    ctxt.push_var(out);
    Ok(())
}

#[derive(Debug, Clone)]
pub(crate) struct TableGrowInstruction {
    table_idx: TableIdx,
    size: VariableID,
    value_to_fill: VariableID,
    out1: VariableID,
}

impl Instruction for TableGrowInstruction {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.write_instruction_type(InstructionType::Table(TableInstructionCategory::Grow));
        o.write_immediate(self.table_idx);
        o.write_variable(self.size);
        o.write_variable(self.value_to_fill);
        o.write_variable(self.out1);
    }

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

pub(crate) fn table_grow(
    ctxt: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let table_idx = TableIdx::parse(i)?;
    let table = match ctxt.module.tables.get(table_idx as usize) {
        Some(table) => table,
        None => {
            ctxt.poison::<()>(ValidationError::Msg(format!(
                "table with id {} not found",
                table_idx
            )));
            return Ok(());
        }
    };
    let table_type = ValType::Reference(table.get_ref_type());

    let size = ctxt.pop_var_with_type(&ValType::Number(NumType::I32));
    let value_to_fill = ctxt.pop_var_with_type(&table_type);
    match value_to_fill.type_ {
        ValType::Reference(_) => {}
        _ => ctxt.poison(ValidationError::Msg(format!(
            "type {:?} of value supplied to table.grow instruction does not match table type {:?}.",
            value_to_fill.type_, table_type
        ))),
    }
    let out = ctxt.create_var(ValType::Number(NumType::I32));
    o.write(TableGrowInstruction {
        table_idx,
        size: size.id,
        value_to_fill: value_to_fill.id,
        out1: out.id,
    });
    ctxt.push_var(out);
    Ok(())
}

#[derive(Debug, Clone)]
pub(crate) struct TableSizeInstruction {
    table_idx: TableIdx,
    out1: VariableID,
}

impl Instruction for TableSizeInstruction {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.write_instruction_type(InstructionType::Table(TableInstructionCategory::Size));
        o.write_immediate(self.table_idx);
        o.write_variable(self.out1);
    }

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

pub(crate) fn table_size(
    ctxt: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let table_idx = TableIdx::parse(i)?;
    if table_idx as usize > ctxt.module.tables.len() {
        ctxt.poison(ValidationError::Msg(format!(
            "table with id {} not found",
            table_idx
        )))
    }
    let out = ctxt.create_var(ValType::Number(NumType::I32));
    o.write(TableSizeInstruction {
        table_idx,
        out1: out.id,
    });
    ctxt.push_var(out);
    Ok(())
}

#[derive(Debug, Clone)]
pub(crate) struct TableFillInstruction {
    table_idx: TableIdx,
    i: VariableID,
    n: VariableID,
    ref_value: VariableID,
}

impl Instruction for TableFillInstruction {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.write_instruction_type(InstructionType::Table(TableInstructionCategory::Fill));
        o.write_immediate(self.table_idx);
        o.write_variable(self.i);
        o.write_variable(self.n);
        o.write_variable(self.ref_value);
    }

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

pub(crate) fn table_fill(
    ctxt: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let table_idx = TableIdx::parse(i)?;
    let n = ctxt.pop_var_with_type(&ValType::Number(NumType::I32));
    let ref_value = ctxt.pop_var();
    let i = ctxt.pop_var_with_type(&ValType::Number(NumType::I32));

    // validate
    let ref_value_type = match ref_value.type_ {
        ValType::Reference(ref_type) => ref_type,
        _ => ctxt.poison(ValidationError::Msg(format!(
            "table value to set must be of reference type, but got {:?}",
            ref_value.type_
        ))),
    };
    match ctxt
        .module
        .tables
        .get(table_idx as usize)
        .map(Table::get_ref_type)
    {
        Some(table_type) => {
            if ref_value_type != table_type {
                ctxt.poison(ValidationError::Msg(format!(
                    "table value to set must be of reference type {:?}, but got {:?}",
                    table_type, ref_value_type
                )))
            }
        }
        None => ctxt.poison(ValidationError::Msg(format!(
            "table with id {} does not exist",
            table_idx
        ))),
    }

    o.write(TableFillInstruction {
        table_idx,
        i: i.id,
        n: n.id,
        ref_value: ref_value.id,
    });
    Ok(())
}

#[derive(Debug, Clone)]
pub(crate) struct TableCopyInstruction {
    table_idx_x: TableIdx,
    table_idx_y: TableIdx,
    n: VariableID,
    s: VariableID,
    d: VariableID,
}

impl Instruction for TableCopyInstruction {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.write_instruction_type(InstructionType::Table(TableInstructionCategory::Copy));
        o.write_immediate(self.table_idx_x);
        o.write_immediate(self.table_idx_y);
        o.write_variable(self.n);
        o.write_variable(self.s);
        o.write_variable(self.d);
    }

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

pub(crate) fn table_copy(
    ctxt: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let table_idx_x = TableIdx::parse(i)?;
    let table_idx_y = TableIdx::parse(i)?;
    let n = ctxt.pop_var_with_type(&ValType::Number(NumType::I32));
    let s = ctxt.pop_var_with_type(&ValType::Number(NumType::I32));
    let d = ctxt.pop_var_with_type(&ValType::Number(NumType::I32));

    // validate
    let table_type_x = match ctxt
        .module
        .tables
        .get(table_idx_x as usize)
        .map(Table::get_ref_type)
    {
        Some(table_type) => table_type,
        None => ctxt.poison(ValidationError::Msg(format!(
            "table with id {} does not exist",
            table_idx_x
        ))),
    };

    let table_type_y = match ctxt
        .module
        .tables
        .get(table_idx_y as usize)
        .map(Table::get_ref_type)
    {
        Some(table_type) => table_type,
        None => ctxt.poison(ValidationError::Msg(format!(
            "table with id {} does not exist",
            table_idx_y
        ))),
    };

    if table_type_x != table_type_y {
        ctxt.poison(ValidationError::Msg(format!(
            "table types must match, but got {:?} and {:?}",
            table_type_x, table_type_y
        )))
    }

    o.write(TableCopyInstruction {
        table_idx_x,
        table_idx_y,
        n: n.id,
        s: s.id,
        d: d.id,
    });
    Ok(())
}

#[derive(Debug, Clone)]
pub(crate) struct TableInitInstruction {
    table_idx: TableIdx,
    elem_idx: ElemIdx,
    n: VariableID,
    s: VariableID,
    d: VariableID,
}

impl Instruction for TableInitInstruction {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.write_instruction_type(InstructionType::Table(TableInstructionCategory::Init));
        o.write_immediate(self.table_idx);
        o.write_immediate(self.elem_idx);
        o.write_variable(self.n);
        o.write_variable(self.s);
        o.write_variable(self.d);
    }

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

pub(crate) fn table_init(
    ctxt: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let elem_idx = ElemIdx::parse(i)?;
    let table_idx = TableIdx::parse(i)?;
    let n = ctxt.pop_var_with_type(&ValType::Number(NumType::I32));
    let s = ctxt.pop_var_with_type(&ValType::Number(NumType::I32));
    let d = ctxt.pop_var_with_type(&ValType::Number(NumType::I32));

    // validate
    let table_type = match ctxt
        .module
        .tables
        .get(table_idx as usize)
        .map(Table::get_ref_type)
    {
        Some(table_type) => table_type,
        None => ctxt.poison(ValidationError::Msg(format!(
            "table with id {} does not exist",
            table_idx
        ))),
    };
    let elem_type = match ctxt.module.elements.get(elem_idx as usize) {
        Some(Element {
            type_: elem_type, ..
        }) => *elem_type,
        None => ctxt.poison(ValidationError::Msg(format!(
            "elem with id {} does not exist",
            elem_idx
        ))),
    };
    if table_type != elem_type {
        ctxt.poison(ValidationError::Msg(format!(
            "table type {:?} must match elem type {:?}",
            table_type, elem_type
        )))
    }

    o.write(TableInitInstruction {
        table_idx,
        elem_idx,
        n: n.id,
        s: s.id,
        d: d.id,
    });
    Ok(())
}

#[derive(Debug, Clone)]
pub(crate) struct ElemDropInstruction {
    elem_idx: ElemIdx,
}

impl Instruction for ElemDropInstruction {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.write_instruction_type(InstructionType::Table(TableInstructionCategory::Drop));
        o.write_immediate(self.elem_idx);
    }

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

pub(crate) fn elem_drop(
    ctxt: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let elem_idx = ElemIdx::parse(i)?;

    // validate
    if ctxt.module.elements.get(elem_idx as usize).is_none() {
        ctxt.poison(ValidationError::Msg(format!(
            "elem with idx {} does not exist",
            elem_idx
        )))
    }

    o.write(ElemDropInstruction { elem_idx });
    Ok(())
}
