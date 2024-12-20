use super::*;
use crate::parsable::Parse;
use module::objects::element::Element;
use wasm_types::*;

pub(crate) fn table_set(
    ctxt: &mut Context,
    i: &mut WasmBinaryReader,
    o: &mut dyn InstructionConsumer,
) -> ParseResult {
    let table_idx = TableIdx::parse(i)?;
    if table_idx as usize >= ctxt.module.tables.len() {
        return Ok(ctxt.poison::<()>(ValidationError::Msg(format!(
            "table with id {table_idx} not found"
        ))));
    }
    let table = &ctxt.module.tables[table_idx as usize];
    let value_to_set = ctxt.pop_var();
    match value_to_set.type_ {
        ValType::Reference(rt) if rt == table.r#type.ref_type => {}
        _ => ctxt.poison(ValidationError::Msg(
            "table value to set must be of reference type".into(),
        )),
    }
    let idx = ctxt.pop_var_with_type(ValType::i32());
    o.write_table_set(TableSetInstruction {
        table_idx,
        in1: value_to_set.id,
        idx: idx.id,
        input_type: value_to_set.type_,
    });
    Ok(())
}

pub(crate) fn table_get(
    ctxt: &mut Context,
    i: &mut WasmBinaryReader,
    o: &mut dyn InstructionConsumer,
) -> ParseResult {
    let table_idx = TableIdx::parse(i)?;
    let table = match ctxt.module.tables.get(table_idx as usize) {
        Some(table) => table,
        None => {
            ctxt.poison::<()>(ValidationError::Msg(format!(
                "table with id {table_idx} not found",
            )));
            return Ok(());
        }
    };
    let table_ref_type = table.r#type.ref_type;
    let idx = ctxt.pop_var_with_type(ValType::i32());
    let out = ctxt.create_var(ValType::Reference(table_ref_type));
    o.write_table_get(TableGetInstruction {
        table_idx,
        out1: out.id,
        idx: idx.id,
    });
    ctxt.push_var(out);
    Ok(())
}

pub(crate) fn table_grow(
    ctxt: &mut Context,
    i: &mut WasmBinaryReader,
    o: &mut dyn InstructionConsumer,
) -> ParseResult {
    let table_idx = TableIdx::parse(i)?;
    let table = match ctxt.module.tables.get(table_idx as usize) {
        Some(table) => table,
        None => {
            ctxt.poison::<()>(ValidationError::Msg(format!(
                "table with id {table_idx} not found",
            )));
            return Ok(());
        }
    };
    let table_type = ValType::Reference(table.r#type.ref_type);

    let size = ctxt.pop_var_with_type(ValType::i32());
    let value_to_fill = ctxt.pop_var_with_type(table_type);
    match value_to_fill.type_ {
        ValType::Reference(_) => {}
        _ => ctxt.poison(ValidationError::Msg(format!(
            "type {:?} of value supplied to table.grow instruction does not match table type {:?}.",
            value_to_fill.type_, table_type
        ))),
    }
    let out = ctxt.create_var(ValType::i32());
    o.write_table_grow(TableGrowInstruction {
        table_idx,
        size: size.id,
        value_to_fill: value_to_fill.id,
        out1: out.id,
    });
    ctxt.push_var(out);
    Ok(())
}

pub(crate) fn table_size(
    ctxt: &mut Context,
    i: &mut WasmBinaryReader,
    o: &mut dyn InstructionConsumer,
) -> ParseResult {
    let table_idx = TableIdx::parse(i)?;
    if table_idx as usize > ctxt.module.tables.len() {
        ctxt.poison(ValidationError::Msg(format!(
            "table with id {table_idx} not found",
        )))
    }
    let out = ctxt.create_var(ValType::i32());
    o.write_table_size(TableSizeInstruction {
        table_idx,
        out1: out.id,
    });
    ctxt.push_var(out);
    Ok(())
}

pub(crate) fn table_fill(
    ctxt: &mut Context,
    i: &mut WasmBinaryReader,
    o: &mut dyn InstructionConsumer,
) -> ParseResult {
    let table_idx = TableIdx::parse(i)?;
    let n = ctxt.pop_var_with_type(ValType::i32());
    let ref_value = ctxt.pop_var();
    let i = ctxt.pop_var_with_type(ValType::i32());

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
        .map(|t| t.r#type.ref_type)
    {
        Some(table_type) => {
            if ref_value_type != table_type {
                ctxt.poison(ValidationError::Msg(format!(
                    "table value to set must be of reference type {table_type:?}, but got {ref_value_type:?}"
                )))
            }
        }
        None => ctxt.poison(ValidationError::Msg(format!(
            "table with id {table_idx} does not exist"
        ))),
    }

    o.write_table_fill(TableFillInstruction {
        table_idx,
        i: i.id,
        n: n.id,
        ref_value: ref_value.id,
    });
    Ok(())
}

pub(crate) fn table_copy(
    ctxt: &mut Context,
    i: &mut WasmBinaryReader,
    o: &mut dyn InstructionConsumer,
) -> ParseResult {
    let table_idx_x = TableIdx::parse(i)?;
    let table_idx_y = TableIdx::parse(i)?;
    let n = ctxt.pop_var_with_type(ValType::i32());
    let s = ctxt.pop_var_with_type(ValType::i32());
    let d = ctxt.pop_var_with_type(ValType::i32());

    // validate
    let table_type_x = match ctxt
        .module
        .tables
        .get(table_idx_x as usize)
        .map(|t| t.r#type.ref_type)
    {
        Some(table_type) => table_type,
        None => ctxt.poison(ValidationError::Msg(format!(
            "table with id {table_idx_x} does not exist"
        ))),
    };

    let table_type_y = match ctxt
        .module
        .tables
        .get(table_idx_y as usize)
        .map(|t| t.r#type.ref_type)
    {
        Some(table_type) => table_type,
        None => ctxt.poison(ValidationError::Msg(format!(
            "table with id {table_idx_y} does not exist"
        ))),
    };

    if table_type_x != table_type_y {
        ctxt.poison(ValidationError::Msg(format!(
            "table types must match, but got {table_type_x:?} and {table_type_y:?}"
        )))
    }

    o.write_table_copy(TableCopyInstruction {
        table_idx_x,
        table_idx_y,
        n: n.id,
        s: s.id,
        d: d.id,
    });
    Ok(())
}

pub(crate) fn table_init(
    ctxt: &mut Context,
    i: &mut WasmBinaryReader,
    o: &mut dyn InstructionConsumer,
) -> ParseResult {
    let elem_idx = ElemIdx::parse(i)?;
    let table_idx = TableIdx::parse(i)?;
    let n = ctxt.pop_var_with_type(ValType::i32());
    let s = ctxt.pop_var_with_type(ValType::i32());
    let d = ctxt.pop_var_with_type(ValType::i32());

    // validate
    let table_type = match ctxt
        .module
        .tables
        .get(table_idx as usize)
        .map(|t| t.r#type.ref_type)
    {
        Some(table_type) => table_type,
        None => ctxt.poison(ValidationError::Msg(format!(
            "table with id {table_idx} does not exist",
        ))),
    };
    let elem_type = match ctxt.module.elements.get(elem_idx as usize) {
        Some(Element {
            type_: elem_type, ..
        }) => *elem_type,
        None => ctxt.poison(ValidationError::Msg(format!(
            "elem with id {elem_idx} does not exist",
        ))),
    };
    if table_type != elem_type {
        ctxt.poison(ValidationError::Msg(format!(
            "table type {table_type:?} must match elem type {elem_type:?}",
        )))
    }

    o.write_table_init(TableInitInstruction {
        table_idx,
        elem_idx,
        n: n.id,
        s: s.id,
        d: d.id,
    });
    Ok(())
}

pub(crate) fn elem_drop(
    ctxt: &mut Context,
    i: &mut WasmBinaryReader,
    o: &mut dyn InstructionConsumer,
) -> ParseResult {
    let elem_idx = ElemIdx::parse(i)?;

    // validate
    if ctxt.module.elements.get(elem_idx as usize).is_none() {
        ctxt.poison(ValidationError::Msg(format!(
            "elem with idx {elem_idx} does not exist",
        )))
    }

    o.write_elem_drop(ElemDropInstruction { elem_idx });
    Ok(())
}
