use crate::{parsable::Parse, wasm_stream_reader::WasmBinaryReader, ParseResult, ParserError};
use context::Context;
use function_builder::FunctionBuilder;
use module::{
    basic_block::BasicBlock,
    instructions::VariableID,
    objects::{function::FunctionIR, instruction::ControlInstruction},
    ModuleMetadata,
};
use parse_basic_blocks::{parse_basic_blocks, validate_and_extract_result_from_stack, Label};
use resource_buffer::ResourceBuffer;
use smallvec::SmallVec;
use stack::ParserStack;
use std::sync::atomic::{AtomicU32, Ordering};
use wasm_types::{FuncIdx, ValType};

pub(crate) mod context;
pub(crate) mod function_builder;
mod opcode_tbl;
pub(crate) mod parse_basic_blocks;
pub(crate) mod stack;

pub(crate) struct FunctionIRParser;

impl FunctionIRParser {
    pub(crate) fn parse_single_function(
        buffer: &ResourceBuffer,
        function_idx: FuncIdx,
        module: &ModuleMetadata,
    ) -> Result<FunctionIR, ParserError> {
        let binary_source = buffer.get()?;
        let mut binary_source = WasmBinaryReader::new(&binary_source);

        // spool to function code / restrict source to function code
        let unparsed_mem = match module.functions[function_idx as usize].get_unparsed_mem() {
            Some(mem) => mem,
            None => return Err(ParserError::MissingFunctionImplementation(function_idx)),
        };
        binary_source.advance(unparsed_mem.offset);
        binary_source.set_limit(unparsed_mem.offset + unparsed_mem.size);

        let mut function = FunctionIR::default();

        let type_idx = module.functions[function_idx as usize].type_idx;
        let function_type = module.function_types[type_idx as usize];

        let stack = ParserStack::new();
        let var_count: AtomicU32 = AtomicU32::new(0);

        let num_locals = binary_source.read_leb128::<u32>()?;
        let mut num_expanded_locals: u64 = 0;
        let local_prototypes = (0..num_locals)
            .map(|_| {
                let count = binary_source.read_leb128::<u32>()?;
                let val_type = ValType::parse(&mut binary_source)?;
                num_expanded_locals = num_expanded_locals.saturating_add(count as u64);
                Ok::<(u32, ValType), ParserError>((count, val_type))
            })
            .collect::<Result<Vec<(u32, ValType)>, _>>()?;
        if num_expanded_locals > u32::MAX as u64 {
            return Err(ParserError::Msg("too many locals".into()));
        }

        {
            function.locals =
                Vec::with_capacity(function_type.num_params() + num_expanded_locals as usize);
            for param_type in function_type.params_iter() {
                function.locals.push(param_type);
            }
            let mut expanded_locals = local_prototypes
                .into_iter()
                .flat_map(|(count, val_type)| (0..count).map(move |_| val_type))
                .collect();
            function.locals.append(&mut expanded_locals);
        }

        let mut ctxt = Context {
            module: &module,
            stack,
            func: &function,
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
        parse_basic_blocks(&mut binary_source, &mut ctxt, &mut labels, &mut builder)?;

        // insert last basic block that always returns from function (jump target for function scope label)
        if !matches!(
            builder.current_bb_terminator(),
            ControlInstruction::Unreachable | ControlInstruction::Return
        ) {
            let _: SmallVec<[VariableID; 0]> =
                validate_and_extract_result_from_stack(&mut ctxt, &function_type.results(), false);
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

        function.num_vars = ctxt.var_count.load(Ordering::Relaxed);
        function.bbs = builder.finalize();
        Ok(function)
    }

    pub fn parse_all_functions(
        module: &mut ModuleMetadata,
        buffer: &ResourceBuffer,
    ) -> ParseResult {
        for func_idx in 0..module.functions.len() {
            let function = &module.functions[func_idx];
            if function.get_import().is_some() || function.get_ir().is_some() {
                continue;
            }

            let function_ir = Self::parse_single_function(buffer, func_idx as FuncIdx, module)?;
            module.functions[func_idx].add_ir(function_ir);
        }
        Ok(())
    }
}
