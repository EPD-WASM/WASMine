use super::parsable::{Parse, ParseWithContext};
use super::ParseResult;
use super::{error::ParserError, wasm_stream_reader::WasmBinaryReader};
use module::objects::data::{Data, DataMode};
use module::objects::{
    element::Element,
    export::{Export, WasmExports},
    expression::ConstantExpression,
    function::{Function, FunctionImport},
    global::Global,
    import::Import,
    memory::Memory,
    table::Table,
    value::ConstantValue,
};
use module::ModuleMetadata;
use std::vec;
use wasm_types::{FuncIdx, FuncType, ImportDesc, MemIdx, Name, Section, TypeIdx};

const WASM_MODULE_PREAMBLE: &[u8] = b"\0asm";
const WASM_MODULE_VERSION: u32 = 1;

pub struct ModuleParser<'a> {
    pub(crate) module: &'a mut ModuleMetadata,
    pub(crate) is_complete: bool,
    pub(crate) next_empty_function: FuncIdx,
}

impl<'a> ModuleParser<'a> {
    pub(crate) fn parse_module(&mut self, i: &mut WasmBinaryReader) -> ParseResult {
        for magic_byte in WASM_MODULE_PREAMBLE {
            if i.read_byte()? != *magic_byte {
                return Err(ParserError::Msg("invalid WASM module preamble".into()));
            }
        }

        let version = i.read_u32()?;
        if version != WASM_MODULE_VERSION {
            return Err(ParserError::Msg(format!(
                "invalid WASM module version \"{version}\"",
            )));
        }

        let mut last_section_number = -1;
        loop {
            let section = match Section::parse(i) {
                Ok(b) => b,
                Err(_) if i.eof()? => break,
                Err(e) => return Err(e),
            };

            if section == Section::Custom {
                self.parse_custom_section(i)?;
                continue;
            }

            if last_section_number >= Section::Data as i32 {
                return Err(ParserError::Msg("invalid additional module section".into()));
            }

            if section.clone() as i32 <= last_section_number {
                return Err(ParserError::Msg("invalid module section order".into()));
            }

            match section {
                Section::Type => self.parse_type_section(i)?,
                Section::Import => self.parse_import_section(i)?,
                Section::Function => self.parse_function_section(i)?,
                Section::Table => self.parse_table_section(i)?,
                Section::Memory => self.parse_memory_section(i)?,
                Section::Global => self.parse_global_section(i)?,
                Section::Export => self.parse_export_section(i)?,
                Section::Start => self.parse_start_section(i)?,
                Section::Element => self.parse_element_section(i)?,
                Section::DataCount => self.parse_datacount_section(i)?,
                Section::Code => self.parse_code_section(i)?,
                Section::Data => self.parse_data_section(i)?,
                Section::Custom => unreachable!(),
            }
            last_section_number = section as i32;
        }
        if self
            .module
            .functions
            .iter()
            .skip(self.next_empty_function as usize)
            .any(|f| {
                // function has neither unparsed binary nor is an import => was not yet parsed!
                f.get_unparsed_mem().is_none() && f.get_import().is_none()
            })
        {
            return Err(ParserError::Msg("function without code".into()));
        }
        self.is_complete = true;
        Ok(())
    }

    fn parse_custom_section(&mut self, i: &mut WasmBinaryReader) -> ParseResult {
        let section_size = i.read_leb128::<u32>()?;
        let reader_pos_safe = i.pos;
        let name = Name::parse(i)?;
        let name_byte_len = i.pos - reader_pos_safe;

        log::warn!("Skipping parsing of custom section \"{}\"", name);
        for _ in name_byte_len..(section_size as usize) {
            let _ = i.read_byte()?;
        }
        Ok(())
    }

    fn parse_type_section(&mut self, i: &mut WasmBinaryReader) -> ParseResult {
        // drop section size as we currently don't need it
        let _ = i.read_leb128::<u32>()?;
        let num_types = i.read_leb128::<u32>()?;
        let mut parsed_function_types = (0..num_types)
            .map(|_| FuncType::parse(i))
            .collect::<Result<Vec<FuncType>, ParserError>>()?;
        self.module
            .function_types
            .append(&mut parsed_function_types);
        Ok(())
    }

    fn parse_import_section(&mut self, i: &mut WasmBinaryReader) -> ParseResult {
        let _ = i.read_leb128::<u32>()?;
        let num_imports = i.read_leb128::<u32>()?;
        for import_idx in 0..num_imports {
            let mut import = Import::parse(i)?;
            match &mut import.desc {
                ImportDesc::Func(type_idx) => {
                    if *type_idx as usize >= self.module.function_types.len() {
                        return Err(ParserError::Msg("function type index out of bounds".into()));
                    }
                    self.module.functions.push(Function {
                        type_idx: *type_idx,
                        source_import: Some(FunctionImport { import_idx }),
                        ..Default::default()
                    });
                }
                ImportDesc::Table(r#type) => self.module.tables.push(Table {
                    r#type: *r#type,
                    import: true,
                }),
                ImportDesc::Mem(limits) => self.module.memories.push(Memory {
                    limits: *limits,
                    import: true,
                }),
                ImportDesc::Global((r#type, idx)) => {
                    *idx = self.module.globals.len() as u32;
                    self.module.globals.push(Global {
                        r#type: *r#type,
                        import: true,
                        init: ConstantValue::FuncPtr(import_idx),
                    })
                }
            }
            self.module.imports.push(import);
        }
        Ok(())
    }

    fn parse_function_section(&mut self, i: &mut WasmBinaryReader) -> ParseResult {
        let _ = i.read_leb128::<u32>()?;
        let num_functions = i.read_leb128::<u32>()?;
        if num_functions == 0 {
            return Ok(());
        }
        let mut parsed_functions = (0..num_functions)
            .map(|_| TypeIdx::parse(i).map(Function::new))
            .collect::<Result<Vec<Function>, ParserError>>()?;
        let max_func_type: u32 = parsed_functions.iter().map(|f| f.type_idx).max().unwrap();
        if max_func_type as usize >= self.module.function_types.len() {
            return Err(ParserError::Msg("function type index out of bounds".into()));
        }
        self.module.functions.append(&mut parsed_functions);
        Ok(())
    }

    fn parse_table_section(&mut self, i: &mut WasmBinaryReader) -> ParseResult {
        let _ = i.read_leb128::<u32>()?;
        let num_tables = i.read_leb128::<u32>()?;
        let mut parsed_tables = (0..num_tables)
            .map(|_| Table::parse(i))
            .collect::<Result<Vec<Table>, ParserError>>()?;
        self.module.tables.append(&mut parsed_tables);
        Ok(())
    }

    fn parse_memory_section(&mut self, i: &mut WasmBinaryReader) -> ParseResult {
        let _ = i.read_leb128::<u32>()?;
        let num_memories = i.read_leb128::<u32>()?;
        if num_memories == 0 {
            return Ok(());
        }
        if num_memories > 1 || !self.module.memories.is_empty() {
            return Err(ParserError::Msg(
                "multiple memories are not supported".into(),
            ));
        }
        self.module.memories = vec![Memory::parse(i)?];
        Ok(())
    }

    fn parse_global_section(&mut self, i: &mut WasmBinaryReader) -> ParseResult {
        let _ = i.read_leb128::<u32>()?;
        let num_globals = i.read_leb128::<u32>()?;
        let mut parsed_globals = (0..num_globals)
            .map(|_| Global::parse_with_context(i, self.module))
            .collect::<Result<Vec<Global>, ParserError>>()?;
        self.module.globals.append(&mut parsed_globals);
        Ok(())
    }

    fn parse_export_section(&mut self, i: &mut WasmBinaryReader) -> ParseResult {
        // normally, we don't need this. But the custom.wast spec test expects this to be checked ¯\_(ツ)_/¯
        let export_section_len = i.read_leb128::<u32>()?;
        if export_section_len == 0 {
            return Ok(());
        }
        let num_exports = i.read_leb128::<u32>()?;
        let mut parsed_exports = WasmExports::default();
        for _ in 0..num_exports {
            let export = Export::parse(i)?;
            match export {
                Export::Func(e) => {
                    if e.idx as usize >= self.module.functions.len() {
                        return Err(ParserError::Msg("function index out of bounds".into()));
                    }
                    parsed_exports.add_function_export(e)
                }
                Export::Table(e) => parsed_exports.add_table_export(e),
                Export::Mem(e) => parsed_exports.add_memory_export(e),
                Export::Global(e) => parsed_exports.add_global_export(e),
            }
        }
        self.module.exports.append(parsed_exports);
        Ok(())
    }

    fn parse_start_section(&mut self, i: &mut WasmBinaryReader) -> ParseResult {
        let _ = i.read_leb128::<u32>()?;
        if self.module.entry_point.is_some() {
            return Err(ParserError::Msg("multiple start sections".into()));
        }
        self.module.entry_point = Some(FuncIdx::parse(i)?);
        if self.module.entry_point.unwrap() as usize >= self.module.functions.len() {
            return Err(ParserError::StartFunctionDoesNotExist);
        }
        Ok(())
    }

    fn parse_element_section(&mut self, i: &mut WasmBinaryReader) -> ParseResult {
        let _ = i.read_leb128::<u32>()?;
        let num_elements = i.read_leb128::<u32>()?;
        let mut parsed_elements = (0..num_elements)
            .map(|_| Element::parse_with_context(i, &self.module))
            .collect::<Result<Vec<Element>, ParserError>>()?;
        self.module.elements.append(&mut parsed_elements);
        Ok(())
    }

    fn parse_datacount_section(&mut self, i: &mut WasmBinaryReader) -> ParseResult {
        let section_size = i.read_leb128::<u32>()?;
        if section_size == 0 {
            return Ok(());
        }
        if self.module.datacount.is_some() {
            return Err(ParserError::Msg("multiple datacount sections".into()));
        }
        self.module.datacount = Some(i.read_leb128::<u32>()?);
        Ok(())
    }

    fn parse_code_section(&mut self, i: &mut WasmBinaryReader) -> ParseResult {
        let _ = i.read_leb128::<u32>()?;
        let num_remaining_function_defs = i.read_leb128::<u32>()?;
        if num_remaining_function_defs > self.module.functions.len() as u32 {
            return Err(ParserError::Msg(format!(
                "code section size {} larger than function count {}",
                num_remaining_function_defs,
                self.module.functions.len()
            )));
        }
        // we unsafely enable parallel access to self.module.ir.functions.
        // This is safe because we never invalidate the iterator in the following loop.
        let functions_to_parse = unsafe { &*(self as *const Self) }
            .module
            .functions
            .iter()
            .enumerate()
            .skip(self.next_empty_function as usize)
            .filter_map(|(idx, _)| {
                if self
                    .module
                    .functions
                    .get(idx)
                    .and_then(|f| f.get_import())
                    .is_some()
                {
                    None
                } else {
                    Some(idx as FuncIdx)
                }
            })
            .take(num_remaining_function_defs as usize)
            .collect::<Vec<_>>();
        for func_idx in functions_to_parse {
            let function_size = i.read_leb128::<u32>()?;
            self.module.functions[func_idx as usize].add_unparsed_mem(i.pos, function_size);
            i.advance(function_size as usize);
            self.next_empty_function = func_idx + 1;
        }
        Ok(())
    }

    fn parse_data_section(&mut self, i: &mut WasmBinaryReader) -> ParseResult {
        // skip empty datasection
        if self.module.datacount.is_some() && self.module.datacount.unwrap() == 0 {
            return Ok(());
        }
        let data_section_size = i.read_leb128::<u32>()?;
        let expected_reader_pos_after_section = i.pos + data_section_size as usize;

        let num_data = i.read_leb128::<u32>()?;
        if self.module.datacount.is_some() && num_data != self.module.datacount.unwrap() {
            return Err(ParserError::Msg(
                "data section size does not match datacount section value".into(),
            ));
        }
        for _ in 0..num_data {
            let mut data = Data {
                mode: DataMode::Passive,
                init: Vec::new(),
            };
            match i.read_leb128::<u32>()? {
                0 => {
                    data.mode = DataMode::Active {
                        memory: 0,
                        offset: ConstantExpression::parse_with_context(i, &self.module)?
                            .eval(&self.module)?,
                    };
                    data.init = Vec::parse(i)?;
                }
                1 => {
                    data.mode = DataMode::Passive;
                    data.init = Vec::parse(i)?;
                }
                2 => {
                    data.mode = DataMode::Active {
                        memory: MemIdx::parse(i)?,
                        offset: ConstantExpression::parse_with_context(i, &self.module)?
                            .eval(&self.module)?,
                    };
                    data.init = Vec::parse(i)?;
                }
                _ => {
                    return Err(ParserError::Msg(
                        "invalid data segment binary prefix".into(),
                    ))
                }
            }
            self.module.datas.push(data);
        }
        if i.pos != expected_reader_pos_after_section {
            return Err(ParserError::Msg(
                "declared data section size does not match actual section size".into(),
            ));
        }
        Ok(())
    }
}
