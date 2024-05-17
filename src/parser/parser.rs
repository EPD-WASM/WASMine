use super::parsable::{Parse, ParseWithContext};
use super::parse_basic_blocks::{parse_basic_blocks, Label};
use super::{error::ParserError, wasm_stream_reader::WasmStreamReader};
use super::{Context, ParseResult, ParserStack};
use crate::instructions::Variable;
use crate::structs::basic_block::BasicBlock;
use crate::structs::data::{Data, DataMode};
use crate::structs::expression::Expression;
use crate::structs::import::ImportDesc;
use crate::wasm_types::wasm_type::{MemIdx, ValType};
use crate::wasm_types::Name;
use crate::{
    structs::{
        element::Element, export::Export, function::Function, global::Global, import::Import,
        memory::Memory, module::Module, table::Table,
    },
    wasm_types::wasm_type::{FuncIdx, FuncType, Section, TypeIdx},
};
use std::io::{BufReader, Read};
use std::sync::atomic::{AtomicU32, Ordering};
use std::vec;

const WASM_MODULE_PREAMBLE: &[u8] = &[b'\0', b'a', b's', b'm'];
const WASM_MODULE_VERSION: u32 = 1;

#[derive(Debug, Default)]
pub struct Parser {
    pub(crate) module: Module,
    pub(crate) is_complete: bool,
}

impl Parser {
    pub fn parse(mut self, input: impl Read) -> Result<Module, ParserError> {
        let mut reader = WasmStreamReader::new(BufReader::new(input));
        match self.parse_module(&mut reader) {
            Err(e) => Err(ParserError::PositionalError(Box::new(e), reader.pos)),
            _ => Ok(self.module),
        }
    }

    fn parse_module(&mut self, i: &mut WasmStreamReader) -> ParseResult {
        for magic_byte in WASM_MODULE_PREAMBLE {
            if i.read_byte()? != *magic_byte {
                return Err(ParserError::Msg("invalid WASM module preamble".into()));
            }
        }

        let version = i.read_u32()?;
        if version != WASM_MODULE_VERSION {
            return Err(ParserError::Msg(format!(
                "invalid WASM module version \"{}\"",
                version
            )));
        }

        // non-custom sections must appear in module at least once in a certain order defined by the spec.
        // This vector contains the section types in the reverse order, so we can pop the last element an
        // compare it with the read section.
        let mut required_sections = vec![
            Section::Data,
            Section::Code,
            Section::DataCount,
            Section::Element,
            Section::Start,
            Section::Export,
            Section::Global,
            Section::Memory,
            Section::Table,
            Section::Function,
            Section::Import,
            Section::Type,
        ];
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

            if required_sections.is_empty() {
                return Err(ParserError::Msg("invalid additional module section".into()));
            }

            while section != required_sections.pop().unwrap() {
                if required_sections.is_empty() {
                    return Err(ParserError::Msg("invalid module section order".into()));
                }
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
        }
        if self
            .module
            .functions
            .iter()
            .any(|f| f.basic_blocks.is_empty() && !f.import)
        {
            return Err(ParserError::Msg("function without code".into()));
        }
        self.is_complete = true;
        Ok(())
    }

    fn parse_custom_section(&mut self, i: &mut WasmStreamReader) -> ParseResult {
        let section_size = i.read_leb128::<u32>()?;
        let reader_pos_safe = i.pos;
        let name = Name::parse(i)?;
        let name_byte_len = i.pos - reader_pos_safe;

        println!("Skipping parsing of custom section {}", name);
        for _ in name_byte_len..section_size {
            let _ = i.read_byte()?;
        }
        Ok(())
    }

    fn parse_type_section(&mut self, i: &mut WasmStreamReader) -> ParseResult {
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

    fn parse_import_section(&mut self, i: &mut WasmStreamReader) -> ParseResult {
        let _ = i.read_leb128::<u32>()?;
        let num_imports = i.read_leb128::<u32>()?;
        for _ in 0..num_imports {
            let import = Import::parse(i)?;
            match import.desc.clone() {
                ImportDesc::Func(idx) => self.module.functions.push(Function {
                    type_idx: idx,
                    import: true,
                    ..Default::default()
                }),
                ImportDesc::Table(r#type) => self.module.tables.push(Table { r#type }),
                ImportDesc::Mem(r#type) => self.module.memories.push(Memory { r#type }),
                ImportDesc::Global(r#type) => self.module.globals.push(Global {
                    r#type,
                    import: true,
                    value: Expression::default(),
                }),
            }
            self.module.imports.push(import);
        }
        Ok(())
    }

    fn parse_function_section(&mut self, i: &mut WasmStreamReader) -> ParseResult {
        let _ = i.read_leb128::<u32>()?;
        let num_functions = i.read_leb128::<u32>()?;
        if num_functions == 0 {
            return Ok(());
        }
        let mut parsed_functions = (0..num_functions)
            .map(|_| {
                TypeIdx::parse(i).map(|idx| Function {
                    type_idx: idx,
                    locals: Vec::new(),
                    basic_blocks: Vec::new(),
                    import: false,
                })
            })
            .collect::<Result<Vec<Function>, ParserError>>()?;
        let max_func_type = parsed_functions.iter().map(|f| f.type_idx).max().unwrap();
        if max_func_type as usize >= self.module.function_types.len() {
            return Err(ParserError::Msg("function type index out of bounds".into()));
        }
        self.module.functions.append(&mut parsed_functions);
        Ok(())
    }

    fn parse_table_section(&mut self, i: &mut WasmStreamReader) -> ParseResult {
        let _ = i.read_leb128::<u32>()?;
        let num_tables = i.read_leb128::<u32>()?;
        let mut parsed_tables = (0..num_tables)
            .map(|_| Table::parse(i))
            .collect::<Result<Vec<Table>, ParserError>>()?;
        self.module.tables.append(&mut parsed_tables);
        Ok(())
    }

    fn parse_memory_section(&mut self, i: &mut WasmStreamReader) -> ParseResult {
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

    fn parse_global_section(&mut self, i: &mut WasmStreamReader) -> ParseResult {
        let _ = i.read_leb128::<u32>()?;
        let num_globals = i.read_leb128::<u32>()?;
        let mut parsed_globals = (0..num_globals)
            .map(|_| Global::parse_with_context(i, &self.module))
            .collect::<Result<Vec<Global>, ParserError>>()?;
        self.module.globals.append(&mut parsed_globals);
        Ok(())
    }

    fn parse_export_section(&mut self, i: &mut WasmStreamReader) -> ParseResult {
        // normally, we don't need this. But the custom.wast spec test expects this to be checked ¯\_(ツ)_/¯
        let export_section_len = i.read_leb128::<u32>()?;
        if export_section_len == 0 {
            return Ok(());
        }
        let num_exports = i.read_leb128::<u32>()?;
        let mut parsed_exports = (0..num_exports)
            .map(|_| Export::parse(i))
            .collect::<Result<Vec<Export>, ParserError>>()?;
        self.module.exports.append(&mut parsed_exports);
        Ok(())
    }

    fn parse_start_section(&mut self, i: &mut WasmStreamReader) -> ParseResult {
        let _ = i.read_leb128::<u32>()?;
        if self.module.entry_point.is_some() {
            return Err(ParserError::Msg("multiple start sections".into()));
        }
        self.module.entry_point = Some(FuncIdx::parse(i)?);
        Ok(())
    }

    fn parse_element_section(&mut self, i: &mut WasmStreamReader) -> ParseResult {
        let _ = i.read_leb128::<u32>()?;
        let num_elements = i.read_leb128::<u32>()?;
        let mut parsed_elements = (0..num_elements)
            .map(|_| Element::parse_with_context(i, &self.module))
            .collect::<Result<Vec<Element>, ParserError>>()?;
        self.module.elements.append(&mut parsed_elements);
        Ok(())
    }

    fn parse_datacount_section(&mut self, i: &mut WasmStreamReader) -> ParseResult {
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

    fn parse_function(&mut self, i: &mut WasmStreamReader, function_idx: usize) -> ParseResult {
        // we don't need the code size for decoding an thus don't validate it
        let _ = i.read_leb128::<u32>()?;

        let stack = ParserStack::new();
        let var_count: AtomicU32 = AtomicU32::new(0);

        let num_locals = i.read_leb128::<u32>()?;
        let mut num_expanded_locals: u64 = 0;
        let local_prototypes = (0..num_locals)
            .map(|_| {
                let count = i.read_leb128::<u32>()?;
                let val_type = ValType::parse(i)?;
                num_expanded_locals = num_expanded_locals.saturating_add(count as u64);
                Ok::<(u32, ValType), ParserError>((count, val_type))
            })
            .collect::<Result<Vec<(u32, ValType)>, _>>()?;
        if num_expanded_locals > u32::MAX as u64 {
            return Err(ParserError::Msg("too many locals".into()));
        }

        let function_type_idx = self.module.functions[function_idx].type_idx;
        let input_param_types = self.module.function_types[function_type_idx as usize]
            .0
            .clone();
        let mut locals = Vec::with_capacity(input_param_types.len() + num_expanded_locals as usize);
        for param_type in input_param_types {
            locals.push(Variable {
                type_: param_type,
                id: var_count.fetch_add(1, Ordering::Relaxed),
            });
        }
        let mut expaneded_locals = local_prototypes
            .into_iter()
            .flat_map(|(count, val_type)| {
                let id = var_count.fetch_add(count, Ordering::Relaxed);
                (0..count).map(move |_| Variable {
                    type_: val_type,
                    id,
                })
            })
            .collect();
        locals.append(&mut expaneded_locals);
        self.module.functions[function_idx].locals = locals;

        let mut ctxt = Context {
            module: &self.module,
            stack,
            func: &self.module.functions[function_idx],
            var_count,
            poison: None,
        };

        let entry_basic_block = BasicBlock::next_id();
        let function_outer_scope_label = Label {
            bb_id: entry_basic_block,
            result_type: self.module.function_types[function_type_idx as usize]
                .1
                .clone(),
        };
        let mut labels = vec![function_outer_scope_label];
        let parsed_basic_blocks = parse_basic_blocks(i, &mut ctxt, &mut labels, entry_basic_block)?;
        if let Some(poison) = ctxt.poison {
            return Err(poison.into());
        }
        self.module.functions[function_idx].basic_blocks = parsed_basic_blocks;
        Ok(())
    }

    fn get_function_name(&self, function_idx: usize) -> String {
        self.module
            .exports
            .iter()
            .find_map(|export| match export {
                Export {
                    name,
                    desc: crate::structs::export::ExportDesc::Func(idx),
                } if *idx == function_idx as u32 => Some(name),
                _ => None,
            })
            .cloned()
            .unwrap_or("<anonymous>".to_owned())
    }

    fn parse_code_section(&mut self, i: &mut WasmStreamReader) -> ParseResult {
        let _ = i.read_leb128::<u32>()?;
        let mut num_remaining_function_defs = i.read_leb128::<u32>()?;
        if num_remaining_function_defs > self.module.functions.len() as u32 {
            return Err(ParserError::Msg(format!(
                "code section size {} larger than function count {}",
                num_remaining_function_defs,
                self.module.functions.len()
            )));
        }

        let mut function_idx = self
            .module
            .functions
            .iter()
            .position(|f| f.basic_blocks.is_empty())
            .unwrap_or(self.module.functions.len());

        while num_remaining_function_defs > 0 {
            let next_free_function_idx = self
                .module
                .functions
                .iter()
                .enumerate()
                .skip(function_idx)
                .filter(|(_, f)| !f.import && f.basic_blocks.is_empty())
                .map(|(i, _)| i)
                .next();

            if next_free_function_idx.is_none() {
                return Err(ParserError::Msg(
                    "multiple code sections for a single function definition".into(),
                ));
            }

            function_idx = next_free_function_idx.unwrap();

            let mut function_parse_result = self.parse_function(i, function_idx);
            #[cfg(debug_assertions)]
            {
                let function_name = self.get_function_name(function_idx);
                function_parse_result = function_parse_result.map_err(|e| {
                    ParserError::Msg(format!(
                        "Error during parsing of function `{}`: {}",
                        function_name, e
                    ))
                })
            }
            function_parse_result?;

            num_remaining_function_defs -= 1;
        }
        Ok(())
    }

    fn parse_data_section(&mut self, i: &mut WasmStreamReader) -> ParseResult {
        // skip empty datasection
        if self.module.datacount.is_some() && self.module.datacount.unwrap() == 0 {
            return Ok(());
        }
        let data_section_size = i.read_leb128::<u32>()?;
        let expected_reader_pos_after_section = i.pos + data_section_size;

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
                        offset: Expression::parse_with_context(i, &self.module)?,
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
                        offset: Expression::parse_with_context(i, &self.module)?,
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

        // TODO: if I understand the spec correctly, this should not be a necessary check, but the spec tests require it...
        if i.pos != expected_reader_pos_after_section {
            return Err(ParserError::Msg(
                "declared data section size does not match actual section size".into(),
            ));
        }
        Ok(())
    }
}
