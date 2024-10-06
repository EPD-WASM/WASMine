use crate::{parsable::Parse, wasm_stream_reader::WasmBinaryReader, ParseResult, ParserError};
use context::Context;
use function_builder::{FunctionBuilderInterface, FunctionIRBuilder};
use module::{
    instructions::{FunctionIR, VariableID},
    objects::{
        function::{FunctionSource, FunctionUnparsed},
        instruction::ControlInstruction,
    },
    Module, ModuleMetadata,
};
use parse_basic_blocks::{parse_basic_blocks, validate_and_extract_result_from_stack, Label};
use resource_buffer::ResourceBuffer;
use smallvec::SmallVec;
use stack::ParserStack;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    RwLock,
};
use wasm_types::{FuncIdx, ValType};

pub(crate) mod context;
pub(crate) mod function_builder;
mod opcode_tbl;
pub(crate) mod parse_basic_blocks;
pub(crate) mod stack;

pub(crate) struct FunctionParser;

impl FunctionParser {
    pub fn parse_single_function(
        buffer: &ResourceBuffer,
        function_idx: FuncIdx,
        function_unparsed: &FunctionUnparsed,
        module: &ModuleMetadata,
        builder: &mut impl FunctionBuilderInterface,
    ) -> Result<(), ParserError> {
        let binary_source = buffer.get()?;
        let mut binary_source = WasmBinaryReader::new(&binary_source);

        // spool to function code / restrict source to function code
        binary_source.advance(function_unparsed.offset);
        binary_source.set_limit(function_unparsed.offset + function_unparsed.size);

        let type_idx = module.functions[function_idx as usize].type_idx;
        let function_type = module.function_types[type_idx as usize];
        builder.init(function_type);

        builder.begin_locals();
        let num_locals = binary_source.read_leb128::<u32>()?;
        let mut local_defs: SmallVec<[(u32, ValType); 2]> = SmallVec::new();
        let mut num_declared_locals: u64 = 0;
        for _ in 0..num_locals {
            local_defs.push((
                binary_source.read_leb128::<u32>()?,
                ValType::parse(&mut binary_source)?,
            ));
            num_declared_locals += local_defs.last().unwrap().0 as u64;
        }
        if num_declared_locals > u32::MAX as u64 {
            return Err(ParserError::Msg("too many locals".into()));
        }

        let mut ctxt_locals = function_type.params();
        ctxt_locals.reserve(num_locals as usize);
        let mut local_idx = 0;
        for (count, val_type) in local_defs {
            for i in 0..count {
                builder.add_local(local_idx + i, val_type);
                ctxt_locals.push(val_type);
            }
            local_idx += count;
        }
        builder.end_locals();

        let mut ctxt = Context {
            module: &module,
            stack: ParserStack::new(),
            locals: ctxt_locals,
            var_count: AtomicUsize::new(0),
            poison: None,
        };
        let entry_basic_block = builder.reserve_bb();
        let exit_basic_block = builder.reserve_bb();

        builder.set_bb_phi_inputs(exit_basic_block, &mut ctxt, function_type.results_iter());

        let function_scope_label = Label {
            bb_id: exit_basic_block,
            loop_after_bb_id: None,
            loop_after_result_type: None,
            result_type: function_type.results(),
        };
        let mut labels = vec![function_scope_label];
        builder.continue_bb(entry_basic_block);
        parse_basic_blocks(&mut binary_source, &mut ctxt, &mut labels, builder)?;

        // insert last basic block that always returns from function (jump target for function scope label)
        if !matches!(
            builder.current_bb_instrs().peek_terminator(),
            ControlInstruction::Unreachable | ControlInstruction::Return
        ) {
            let _: SmallVec<[VariableID; 0]> =
                validate_and_extract_result_from_stack(&mut ctxt, &function_type.results(), false);
        }
        builder.continue_bb(exit_basic_block);
        builder.terminate_return(builder.current_bb_input_var_ids_get());

        if let Some(poison) = ctxt.poison {
            return Err(poison.into());
        }

        builder.set_var_count(ctxt.var_count.load(Ordering::Relaxed));
        Ok(())
    }

    pub(crate) fn parse_all_functions(module: &Module) -> ParseResult {
        if !module.artifact_registry.read().unwrap().contains_key("ir") {
            module.artifact_registry.write().unwrap().insert(
                "ir".to_string(),
                RwLock::new(Box::new(Vec::<FunctionIR>::new())),
            );
        }
        let artifact_registry = module.artifact_registry.read().unwrap();
        let artifact_ref = artifact_registry.get("ir").unwrap();
        let mut artifact_ref = artifact_ref.write().unwrap();
        let ir = artifact_ref.downcast_mut::<Vec<FunctionIR>>().unwrap();

        ir.resize(module.meta.functions.len(), FunctionIR::default());
        for func_idx in 0..module.meta.functions.len() {
            if ir[func_idx].bbs.len() > 0 {
                continue;
            }

            let function_meta = &module.meta.functions[func_idx];
            match &function_meta.source {
                FunctionSource::Import(_) => continue,
                FunctionSource::Wasm(function_unparsed) => {
                    let mut builder = FunctionIRBuilder::new();
                    Self::parse_single_function(
                        &module.source,
                        func_idx as FuncIdx,
                        function_unparsed,
                        &module.meta,
                        &mut builder,
                    )?;
                    ir[func_idx] = builder.finalize();
                }
            }
        }
        Ok(())
    }
}
