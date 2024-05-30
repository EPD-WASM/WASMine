use super::*;

pub(crate) fn memory_size(
    ctxt: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    if i.read_byte()? != 0 {
        return Err(ParserError::Msg(
            "memory size instruction invalid encoding".into(),
        ));
    }

    let out = ctxt.create_var(ValType::Number(NumType::I32));
    o.write(MemorySizeInstruction { out1: out.id });
    ctxt.push_var(out);
    Ok(())
}

pub(crate) fn memory_grow(
    ctxt: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    if i.read_byte()? != 0 {
        return Err(ParserError::Msg(
            "memory grow instruction invalid encoding".into(),
        ));
    }

    let size_in = ctxt.pop_var_with_type(&ValType::Number(NumType::I32));
    let out = ctxt.create_var(ValType::Number(NumType::I32));
    o.write(MemoryGrowInstruction {
        in1: size_in.id,
        out1: out.id,
    });
    ctxt.push_var(out);
    Ok(())
}

pub(crate) fn memory_copy(
    ctxt: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    if i.read_byte()? != 0 || i.read_byte()? != 0 {
        return Err(ParserError::Msg(
            "memory copy instruction invalid encoding".into(),
        ));
    }

    let n = ctxt.pop_var_with_type(&ValType::Number(NumType::I32));
    let s = ctxt.pop_var_with_type(&ValType::Number(NumType::I32));
    let d = ctxt.pop_var_with_type(&ValType::Number(NumType::I32));
    o.write(MemoryCopyInstruction {
        n: n.id,
        s: s.id,
        d: d.id,
    });
    Ok(())
}

pub(crate) fn memory_fill(
    ctxt: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    if i.read_byte()? != 0 {
        return Err(ParserError::Msg(
            "memory fill instruction invalid encoding".into(),
        ));
    }

    let n = ctxt.pop_var_with_type(&ValType::Number(NumType::I32));
    let val = ctxt.pop_var_with_type(&ValType::Number(NumType::I32));
    let d = ctxt.pop_var_with_type(&ValType::Number(NumType::I32));
    o.write(MemoryFillInstruction {
        n: n.id,
        val: val.id,
        d: d.id,
    });
    Ok(())
}

pub(crate) fn memory_init(
    ctxt: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let data_idx: u32 = DataIdx::parse(i)?;

    if let Some(data_count) = ctxt.module.datacount {
        if data_idx >= data_count {
            ctxt.poison(ValidationError::Msg(
                "memory init instruction data index out of bounds".into(),
            ))
        }
    } else {
        ctxt.poison(ValidationError::Msg(
            "memory init instruction without data section".into(),
        ))
    }

    if i.read_byte()? != 0 {
        return Err(ParserError::Msg(
            "memory init instruction invalid encoding".into(),
        ));
    }
    let n = ctxt.pop_var_with_type(&ValType::Number(NumType::I32));
    let s = ctxt.pop_var_with_type(&ValType::Number(NumType::I32));
    let d = ctxt.pop_var_with_type(&ValType::Number(NumType::I32));

    o.write(MemoryInitInstruction {
        data_idx,
        n: n.id,
        s: s.id,
        d: d.id,
    });
    Ok(())
}

pub(crate) fn data_drop(
    ctxt: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let data_idx = DataIdx::parse(i)?;
    if let Some(data_count) = ctxt.module.datacount {
        if data_idx >= data_count {
            ctxt.poison(ValidationError::Msg(
                "memory init instruction data index out of bounds".into(),
            ))
        }
    } else {
        ctxt.poison(ValidationError::Msg(
            "memory init instruction without data section".into(),
        ))
    }

    o.write(DataDropInstruction { data_idx });
    Ok(())
}
