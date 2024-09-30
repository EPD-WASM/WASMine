use super::*;

pub(crate) fn memory_size(
    ctxt: &mut Context,
    i: &mut WasmBinaryReader,
    o: &mut dyn InstructionConsumer,
) -> ParseResult {
    if i.read_byte()? != 0 {
        return Err(ParserError::Msg(
            "memory size instruction invalid encoding".into(),
        ));
    }

    let out = ctxt.create_var(ValType::i32());
    o.write_memory_size(MemorySizeInstruction { out1: out.id });
    ctxt.push_var(out);
    Ok(())
}

pub(crate) fn memory_grow(
    ctxt: &mut Context,
    i: &mut WasmBinaryReader,
    o: &mut dyn InstructionConsumer,
) -> ParseResult {
    if i.read_byte()? != 0 {
        return Err(ParserError::Msg(
            "memory grow instruction invalid encoding".into(),
        ));
    }

    let size_in = ctxt.pop_var_with_type(ValType::i32());
    let out = ctxt.create_var(ValType::i32());
    o.write_memory_grow(MemoryGrowInstruction {
        in1: size_in.id,
        out1: out.id,
    });
    ctxt.push_var(out);
    Ok(())
}

pub(crate) fn memory_copy(
    ctxt: &mut Context,
    i: &mut WasmBinaryReader,
    o: &mut dyn InstructionConsumer,
) -> ParseResult {
    if i.read_byte()? != 0 || i.read_byte()? != 0 {
        return Err(ParserError::Msg(
            "memory copy instruction invalid encoding".into(),
        ));
    }
    if ctxt.module.memories.is_empty() {
        return Err(ParserError::Msg("unknown memory 0".into()));
    }

    let n = ctxt.pop_var_with_type(ValType::i32());
    let s = ctxt.pop_var_with_type(ValType::i32());
    let d = ctxt.pop_var_with_type(ValType::i32());
    o.write_memory_copy(MemoryCopyInstruction {
        n: n.id,
        s: s.id,
        d: d.id,
    });
    Ok(())
}

pub(crate) fn memory_fill(
    ctxt: &mut Context,
    i: &mut WasmBinaryReader,
    o: &mut dyn InstructionConsumer,
) -> ParseResult {
    if i.read_byte()? != 0 {
        return Err(ParserError::Msg(
            "memory fill instruction invalid encoding".into(),
        ));
    }
    if ctxt.module.memories.is_empty() {
        return Err(ParserError::Msg("unknown memory 0".into()));
    }

    let n = ctxt.pop_var_with_type(ValType::i32());
    let val = ctxt.pop_var_with_type(ValType::i32());
    let d = ctxt.pop_var_with_type(ValType::i32());
    o.write_memory_fill(MemoryFillInstruction {
        n: n.id,
        val: val.id,
        d: d.id,
    });
    Ok(())
}

pub(crate) fn memory_init(
    ctxt: &mut Context,
    i: &mut WasmBinaryReader,
    o: &mut dyn InstructionConsumer,
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
    let n = ctxt.pop_var_with_type(ValType::i32());
    let s = ctxt.pop_var_with_type(ValType::i32());
    let d = ctxt.pop_var_with_type(ValType::i32());

    o.write_memory_init(MemoryInitInstruction {
        data_idx,
        n: n.id,
        s: s.id,
        d: d.id,
    });
    Ok(())
}

pub(crate) fn data_drop(
    ctxt: &mut Context,
    i: &mut WasmBinaryReader,
    o: &mut dyn InstructionConsumer,
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

    o.write_data_drop(DataDropInstruction { data_idx });
    Ok(())
}
