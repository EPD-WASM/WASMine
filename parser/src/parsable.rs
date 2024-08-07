use super::{
    error::ParserError, parse_basic_blocks::parse_basic_blocks,
    wasm_stream_reader::WasmStreamReader,
};
use crate::{context::Context, function_builder::FunctionBuilder};
use ir::{
    function::Function,
    structs::{
        element::{ElemMode, Element, ElementInit},
        export::{Export, FuncExport, GlobalExport, MemoryExport, TableExport},
        expression::{ConstantExpression, ConstantExpressionError},
        global::Global,
        import::Import,
        memory::{MemArg, Memory},
        module::Module,
        table::Table,
        value::ConstantValue,
    },
};
use wasm_types::{
    BlockType, FuncIdx, FuncType, FuncTypeBuilder, GlobalIdx, GlobalType, ImportDesc, Limits,
    MemIdx, MemType, Name, NumType, RefType, Section, TableIdx, TableType, TypeIdx, ValType,
};

pub(crate) trait Parse {
    fn parse(i: &mut WasmStreamReader) -> Result<Self, ParserError>
    where
        Self: std::marker::Sized;
}

impl Parse for FuncType {
    fn parse(i: &mut WasmStreamReader) -> Result<FuncType, ParserError> {
        let prefix = i.read_byte()?;
        if prefix != 0x60 {
            return Err(ParserError::Msg(format!(
                "invalid function type prefix. 0x{:x} at position {:x} != 0x60",
                prefix, i.pos,
            )));
        }
        let mut builder = FuncTypeBuilder::new();
        for _ in 0..i.read_leb128::<u32>()? {
            builder = builder.add_param(ValType::parse(i)?);
        }
        for _ in 0..i.read_leb128::<u32>()? {
            builder = builder.add_result(ValType::parse(i)?);
        }
        Ok(builder.finish())
    }
}

impl Parse for ValType {
    fn parse(i: &mut WasmStreamReader) -> Result<ValType, ParserError> {
        let prefix = i.read_byte()?;
        match prefix {
            0x7F => Ok(ValType::Number(NumType::I32)),
            0x7E => Ok(ValType::Number(NumType::I64)),
            0x7D => Ok(ValType::Number(NumType::F32)),
            0x7C => Ok(ValType::Number(NumType::F64)),

            0x7B => Ok(ValType::VecType),

            0x70 => Ok(ValType::Reference(RefType::FunctionReference)),
            0x6F => Ok(ValType::Reference(RefType::ExternReference)),

            _ => Err(ParserError::Msg(format!("invalid value type prefix. Expected 0x7F, 0x7E, 0x7D, 0x7C, 0x7B, 0x70, or 0x6F, got 0x{} at position {:x}", prefix, i.pos))),
        }
    }
}

impl Parse for Name {
    fn parse(i: &mut WasmStreamReader) -> Result<Name, ParserError> {
        let len = i.read_leb128::<u32>()?;
        let byte_string = (0..len)
            .map(|_| i.read_byte())
            .collect::<Result<Vec<u8>, ParserError>>()?;
        let s = String::from_utf8(byte_string).map_err(|_| {
            ParserError::Msg("failed to convert binary identifier to utf-8 string".into())
        })?;
        Ok(s)
    }
}

impl Parse for u32 {
    fn parse(i: &mut WasmStreamReader) -> Result<TypeIdx, ParserError> {
        i.read_leb128()
    }
}

impl Parse for Limits {
    fn parse(i: &mut WasmStreamReader) -> Result<Limits, ParserError> {
        match i.read_byte()? {
            0x00 => Ok(Limits {
                min: i.read_leb128()?,
                max: None,
            }),
            0x01 => {
                let min = i.read_leb128()?;
                let max = i.read_leb128()?;
                if max < min {
                    return Err(ParserError::LimitsMinimumGreaterThanMaximum);
                }
                Ok(Limits {
                    min,
                    max: Some(max),
                })
            }
            _ => Err(ParserError::Msg("invalid limit type prefix".into())),
        }
    }
}

impl Parse for RefType {
    fn parse(i: &mut WasmStreamReader) -> Result<RefType, ParserError> {
        match i.read_byte()? {
            0x70 => Ok(RefType::FunctionReference),
            0x6F => Ok(RefType::ExternReference),
            _ => Err(ParserError::Msg("invalid reference type prefix".into())),
        }
    }
}

impl Parse for TableType {
    fn parse(i: &mut WasmStreamReader) -> Result<TableType, ParserError> {
        let ref_type = RefType::parse(i)?;
        let lim = Limits::parse(i)?;
        Ok(TableType { ref_type, lim })
    }
}

impl Parse for GlobalType {
    fn parse(i: &mut WasmStreamReader) -> Result<GlobalType, ParserError> {
        let value_type = ValType::parse(i)?;
        match i.read_byte()? {
            0x00 => Ok(GlobalType::Const(value_type)),
            0x01 => Ok(GlobalType::Mut(value_type)),
            _ => Err(ParserError::Msg("invalid global type mut prefix".into())),
        }
    }
}

impl Parse for ImportDesc {
    fn parse(i: &mut WasmStreamReader) -> Result<ImportDesc, ParserError> {
        match i.read_byte()? {
            0x00 => Ok(ImportDesc::Func(TypeIdx::parse(i)?)),
            0x01 => Ok(ImportDesc::Table(TableType::parse(i)?)),
            0x02 => Ok(ImportDesc::Mem(MemType::parse(i)?)),
            0x03 => Ok(ImportDesc::Global((GlobalType::parse(i)?, u32::MAX))),
            _ => Err(ParserError::Msg("invalid import description prefix".into())),
        }
    }
}

impl Parse for Import {
    fn parse(i: &mut WasmStreamReader) -> Result<Import, ParserError> {
        let module = Name::parse(i)?;
        let name = Name::parse(i)?;
        let desc = ImportDesc::parse(i)?;
        Ok(Import { module, name, desc })
    }
}

impl Parse for Table {
    fn parse(i: &mut WasmStreamReader) -> Result<Table, ParserError> {
        Ok(Table {
            r#type: TableType::parse(i)?,
            import: false,
        })
    }
}

impl Parse for Memory {
    fn parse(i: &mut WasmStreamReader) -> Result<Memory, ParserError> {
        Ok(Memory {
            limits: Limits::parse(i)?,
            import: false,
        })
    }
}

impl Parse for Export {
    fn parse(i: &mut WasmStreamReader) -> Result<Export, ParserError> {
        let name = Name::parse(i)?;
        // parse ExportDesc
        match i.read_byte()? {
            0x00 => Ok(Export::Func(FuncExport {
                name,
                idx: FuncIdx::parse(i)?,
            })),
            0x01 => Ok(Export::Table(TableExport {
                name,
                idx: TableIdx::parse(i)?,
            })),
            0x02 => Ok(Export::Mem(MemoryExport {
                name,
                idx: MemIdx::parse(i)?,
            })),
            0x03 => Ok(Export::Global(GlobalExport {
                name,
                idx: GlobalIdx::parse(i)?,
            })),
            _ => Err(ParserError::Msg("invalid export description prefix".into())),
        }
    }
}

impl ParseWithContext for Element {
    fn parse_with_context(i: &mut WasmStreamReader, m: &Module) -> Result<Element, ParserError> {
        let code = i.read_leb128::<u32>()?;
        if code > 7 {
            return Err(ParserError::Msg("invalid element prefix code".into()));
        }

        let table_idx = if code & 0b11 == 2 {
            TableIdx::parse(i)?
        } else {
            0
        };

        let mode = if code & 0b1 != 0 {
            if code & 0b10 != 0 {
                ElemMode::Declarative
            } else {
                ElemMode::Passive
            }
        } else {
            let constant_expr = ConstantExpression::parse_with_context(i, m)?;
            ElemMode::Active {
                table: table_idx,
                offset: constant_expr.eval(m)?,
            }
        };

        if code & 0b100 != 0 {
            let type_ = if code == 4 {
                RefType::FunctionReference
            } else {
                RefType::parse(i)?
            };
            let num_elems = i.read_leb128::<u32>()?;
            let elems = (0..num_elems)
                .map(|_| ConstantExpression::parse_with_context(i, m))
                .flat_map(|r| {
                    r.map(|e| e.eval(m).map_err(ParserError::from))
                        .map_err(|_| ParserError::InvalidEncoding)
                })
                .collect::<Result<Vec<ConstantValue>, ParserError>>()?;
            Ok(Element {
                mode,
                type_,
                init: ElementInit::Final(elems),
            })
        } else {
            let type_ = if code == 0 {
                RefType::FunctionReference
            } else {
                if i.read_byte()? != 0 {
                    return Err(ParserError::Msg("invalid elemkind code".into()));
                }
                RefType::FunctionReference
            };
            let num_elems = i.read_leb128::<u32>()?;
            let elems = (0..num_elems)
                .map(|_| FuncIdx::parse(i))
                .collect::<Result<Vec<FuncIdx>, ParserError>>()?;
            Ok(Element {
                mode,
                type_,
                init: ElementInit::Unresolved(elems),
            })
        }
    }
}

impl Parse for Vec<u8> {
    fn parse(i: &mut WasmStreamReader) -> Result<Vec<u8>, ParserError> {
        let len = i.read_leb128::<u32>()?;
        (0..len).map(|_| i.read_byte()).collect()
    }
}

impl ParseWithContext for BlockType {
    // this encoding is (in my opinion) cursed compared to the rest of the spec
    fn parse_with_context(i: &mut WasmStreamReader, m: &Module) -> Result<Self, ParserError>
    where
        Self: std::marker::Sized,
    {
        let next_byte = i.peek_byte()?;
        if next_byte == 0x40 {
            i.read_byte()?;
            return Ok(BlockType::Empty);
        }

        // this is either a value type whose high bit is not set, or a leb128 signed encoded type index
        if ValType::is_valtype_byte(next_byte) {
            return Ok(BlockType::ShorthandFunc(ValType::parse(i)?));
        }

        let type_idx = i.read_leb128::<i64>()? as TypeIdx;
        if type_idx >= m.function_types.len() as u32 {
            return Err(ParserError::Msg("function index out of bounds".to_string()));
        }
        Ok(BlockType::FunctionSig(type_idx))
    }
}

impl Parse for MemArg {
    fn parse(i: &mut WasmStreamReader) -> Result<MemArg, ParserError> {
        let align = i.read_leb128::<u32>()?;
        let offset = i.read_leb128::<u32>()?;
        Ok(MemArg { align, offset })
    }
}

impl Parse for Section {
    fn parse(i: &mut WasmStreamReader) -> Result<Section, ParserError> {
        Ok(match i.read_byte()? {
            0 => Section::Custom,
            1 => Section::Type,
            2 => Section::Import,
            3 => Section::Function,
            4 => Section::Table,
            5 => Section::Memory,
            6 => Section::Global,
            7 => Section::Export,
            8 => Section::Start,
            9 => Section::Element,
            10 => Section::Code,
            11 => Section::Data,
            12 => Section::DataCount,
            i => return Err(ParserError::Msg(format!("invalid section id {}", i))),
        })
    }
}

pub(crate) trait ParseWithContext {
    fn parse_with_context(i: &mut WasmStreamReader, m: &Module) -> Result<Self, ParserError>
    where
        Self: std::marker::Sized;
}

impl ParseWithContext for ConstantExpression {
    // only used for parsing constant expressions
    fn parse_with_context(
        i: &mut WasmStreamReader,
        module: &Module,
    ) -> Result<ConstantExpression, ParserError> {
        let fake_func = match Function::create_empty(0).src {
            ir::function::FunctionSource::Internal(f) => f,
            _ => unreachable!(),
        };
        let mut ctxt = Context::new(module, &fake_func);
        let mut labels = Vec::new();
        let mut builder = FunctionBuilder::new();
        builder.start_bb();
        parse_basic_blocks(i, &mut ctxt, &mut labels, &mut builder)?;
        let mut parsed_init_blocks = builder.finalize();
        if parsed_init_blocks.len() != 1
            || parsed_init_blocks[0].instructions.instruction_storage.len() != 1
        {
            return Err(ConstantExpressionError::Msg(
                "invalid constant expression. Expected a single constant initializer instruction."
                    .into(),
            )
            .into());
        }
        Ok(ConstantExpression {
            expression: parsed_init_blocks.remove(0).instructions,
        })
    }
}

impl ParseWithContext for Global {
    fn parse_with_context(i: &mut WasmStreamReader, m: &Module) -> Result<Self, ParserError> {
        let r#type = GlobalType::parse(i)?;
        let const_expr = ConstantExpression::parse_with_context(i, m)?;
        Ok(Global {
            r#type,
            init: const_expr.eval(m)?,
            import: false,
        })
    }
}
