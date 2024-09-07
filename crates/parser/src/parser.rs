use super::parsable::{Parse, ParseWithContext};
use super::parse_basic_blocks::{parse_basic_blocks, Label};
use super::ParseResult;
use super::{error::ParserError, wasm_stream_reader::WasmStreamReader};
use crate::context::Context;
use crate::function_builder::FunctionBuilder;
use crate::parse_basic_blocks::validate_and_extract_result_from_stack;
use crate::stack::ParserStack;
use ir::basic_block::BasicBlock;
use ir::function::{Function, FunctionImport, FunctionInternal, FunctionSource};
use ir::structs::data::{Data, DataMode};
use ir::structs::export::WasmExports;
use ir::structs::expression::ConstantExpression;
use ir::structs::instruction::ControlInstruction;
use ir::structs::value::ConstantValue;
use ir::structs::{
    element::Element, export::Export, global::Global, import::Import, memory::Memory,
    module::Module, table::Table,
};
use loader::WasmLoader;
use std::io::BufReader;
use std::sync::atomic::{AtomicU32, Ordering};
use std::vec;
use wasm_types::{FuncIdx, FuncType, ImportDesc, MemIdx, Name, Section, TypeIdx, ValType};

#[cfg(debug_assertions)]
use std::io::Write;

const WASM_MODULE_PREAMBLE: &[u8] = b"\0asm";
const WASM_MODULE_VERSION: u32 = 1;

#[derive(Debug, Default)]
pub struct Parser {
    pub(crate) module: Module,
    pub(crate) is_complete: bool,
    pub(crate) next_empty_function: FuncIdx,
}

impl Parser {
    pub fn parse(mut self, loader: WasmLoader) -> Result<Module, ParserError> {
        let input = loader.load()?;
        let mut reader = WasmStreamReader::new(BufReader::new(input));
        match self.parse_module(&mut reader) {
            Err(e) => Err(ParserError::PositionalError(Box::new(e), reader.pos)),
            _ => {
                #[cfg(debug_assertions)]
                {
                    // write parsed module to file as string
                    let mut f = std::fs::File::create("debug_output.parsed").unwrap();
                    f.write_all(format!("{}", self.module).as_bytes()).unwrap();
                }
                Ok(self.module)
            }
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
                "invalid WASM module version \"{version}\"",
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
        if self.module.ir.functions.iter().skip(self.next_empty_function as usize).any(|f| matches!(&f.src, FunctionSource::Internal(FunctionInternal { bbs, .. }) if bbs.is_empty())) {
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

        log::warn!("Skipping parsing of custom section \"{}\"", name);
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
        for import_idx in 0..num_imports {
            let mut import = Import::parse(i)?;
            match &mut import.desc {
                ImportDesc::Func(type_idx) => {
                    if *type_idx as usize >= self.module.function_types.len() {
                        return Err(ParserError::Msg("function type index out of bounds".into()));
                    }
                    self.module.ir.functions.push(Function {
                        type_idx: *type_idx,
                        src: FunctionSource::Import(FunctionImport { import_idx }),
                    })
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

    fn parse_function_section(&mut self, i: &mut WasmStreamReader) -> ParseResult {
        let _ = i.read_leb128::<u32>()?;
        let num_functions = i.read_leb128::<u32>()?;
        if num_functions == 0 {
            return Ok(());
        }
        let mut parsed_functions = (0..num_functions)
            .map(|_| TypeIdx::parse(i).map(Function::create_empty))
            .collect::<Result<Vec<Function>, ParserError>>()?;
        let max_func_type = parsed_functions.iter().map(|f| f.type_idx).max().unwrap();
        if max_func_type as usize >= self.module.function_types.len() {
            return Err(ParserError::Msg("function type index out of bounds".into()));
        }
        self.module.ir.functions.append(&mut parsed_functions);
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
        let mut parsed_exports = WasmExports::default();
        for _ in 0..num_exports {
            let export = Export::parse(i)?;
            match export {
                Export::Func(e) => {
                    if e.idx as usize >= self.module.ir.functions.len() {
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

    fn parse_start_section(&mut self, i: &mut WasmStreamReader) -> ParseResult {
        let _ = i.read_leb128::<u32>()?;
        if self.module.entry_point.is_some() {
            return Err(ParserError::Msg("multiple start sections".into()));
        }
        self.module.entry_point = Some(FuncIdx::parse(i)?);
        if self.module.entry_point.unwrap() as usize >= self.module.ir.functions.len() {
            return Err(ParserError::StartFunctionDoesNotExist);
        }
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

    fn parse_function(&mut self, i: &mut WasmStreamReader, function_idx: FuncIdx) -> ParseResult {
        debug_assert!(!matches!(
            self.module.ir.functions[function_idx as usize].src,
            FunctionSource::Import(..)
        ));
        let type_idx = self.module.ir.functions[function_idx as usize].type_idx;
        let function_type = self.module.function_types[type_idx as usize];

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

        {
            let locals = match &mut self.module.ir.functions[function_idx as usize].src {
                FunctionSource::Internal(f) => &mut f.locals,
                _ => unreachable!(),
            };
            *locals = Vec::with_capacity(function_type.num_params() + num_expanded_locals as usize);
            for param_type in function_type.params_iter() {
                locals.push(param_type);
            }
            let mut expanded_locals = local_prototypes
                .into_iter()
                .flat_map(|(count, val_type)| (0..count).map(move |_| val_type))
                .collect();
            locals.append(&mut expanded_locals);
        }

        let (num_vars, bbs) = {
            let mut ctxt = Context {
                module: &self.module,
                stack,
                func: match &self.module.ir.functions[function_idx as usize].src {
                    FunctionSource::Internal(f) => f,
                    _ => unreachable!(),
                },
                var_count,
                poison: None,
            };
            let mut builder = FunctionBuilder::new();

            let entry_basic_block = BasicBlock::next_id();
            let exit_basic_block = BasicBlock::next_id();

            builder.reserve_bb_with_phis(exit_basic_block, &mut ctxt, function_type.results_iter());

            let function_scope_label = Label {
                bb_id: exit_basic_block,
                loop_after_bb_id: None,
                loop_after_result_type: None,
                result_type: function_type.results(),
            };
            let mut labels = vec![function_scope_label];
            builder.start_bb_with_id(entry_basic_block);
            parse_basic_blocks(i, &mut ctxt, &mut labels, &mut builder)?;

            // insert last basic block that always returns from function (jump target for function scope label)
            if !matches!(
                builder.current_bb_terminator(),
                ControlInstruction::Unreachable | ControlInstruction::Return
            ) {
                let _ = validate_and_extract_result_from_stack(
                    &mut ctxt,
                    &function_type.results(),
                    false,
                );
            }
            builder.continue_bb(exit_basic_block);
            builder.terminate_return(
                builder
                    .current_bb_get()
                    .inputs
                    .iter()
                    .map(|phi| phi.out)
                    .collect(),
            );

            if let Some(poison) = ctxt.poison {
                return Err(poison.into());
            }

            (ctxt.var_count.load(Ordering::Relaxed), builder.finalize())
        };
        match &mut self.module.ir.functions[function_idx as usize].src {
            FunctionSource::Internal(src) => {
                src.num_vars = num_vars;
                src.bbs = bbs;
            }
            _ => unreachable!(),
        };
        Ok(())
    }

    fn parse_code_section(&mut self, i: &mut WasmStreamReader) -> ParseResult {
        let _ = i.read_leb128::<u32>()?;
        let num_remaining_function_defs = i.read_leb128::<u32>()?;
        if num_remaining_function_defs > self.module.ir.functions.len() as u32 {
            return Err(ParserError::Msg(format!(
                "code section size {} larger than function count {}",
                num_remaining_function_defs,
                self.module.ir.functions.len()
            )));
        }
        // we unsafely enable parallel access to self.module.ir.functions.
        // This is safe because we never invalidate the iterator in the following loop.
        let functions_to_parse = unsafe { &*(self as *const Self) }
            .module
            .ir
            .functions
            .iter()
            .enumerate()
            .skip(self.next_empty_function as usize)
            .filter_map(|(idx, f)| {
                if let FunctionSource::Internal(FunctionInternal { bbs, .. }) = &f.src {
                    if bbs.is_empty() {
                        return Some(idx as FuncIdx);
                    }
                }
                None
            })
            .take(num_remaining_function_defs as usize);
        for func_idx in functions_to_parse {
            self.parse_function(i, func_idx)?;
            self.next_empty_function = func_idx + 1;
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
