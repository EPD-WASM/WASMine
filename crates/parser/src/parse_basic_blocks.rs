use super::{opcode_tbl::LVL1_JMP_TABLE, ParseResult, ValidationError};
use crate::{
    context::Context, error::ParserError, function_builder::FunctionBuilder, stack::ParserStack,
    wasm_stream_reader::WasmStreamReader,
};
use ir::{
    basic_block::{BasicBlock, BasicBlockGlue, BasicBlockID},
    instructions::{Variable, VariableID},
    structs::instruction::ControlInstruction,
};
use wasm_types::BlockType;
use wasm_types::{RefType, ResType, ValType};

struct BTWrapper(BlockType);

impl BTWrapper {
    fn setup_block_stack(&self, ctxt: &mut Context) -> Vec<VariableID> {
        // divide stack into block stack (input params) and remaining stack (output params)
        let input_length = self.block_inputs_count(ctxt);
        ctxt.stack.stash_with_keep(input_length);
        let mut input_vars = Vec::new();
        for (i, input_var) in self.block_inputs(ctxt).enumerate() {
            if ctxt.stack[i].type_ != input_var {
                ctxt.poison(ValidationError::Msg(
                    "mismatched input signature in target label".to_string(),
                ))
            } else {
                input_vars.push(ctxt.stack[i].id);
            }
        }
        input_vars
    }

    fn block_returns(&self, ctxt: &Context) -> Box<dyn Iterator<Item = ValType>> {
        match self.0 {
            BlockType::Empty => Box::new(std::iter::empty()),
            BlockType::ShorthandFunc(val_type) => Box::new(std::iter::once(val_type)),
            BlockType::FunctionSig(func_idx) => {
                Box::new(ctxt.module.function_types[func_idx as usize].results_iter())
            }
        }
    }

    fn block_returns_count(&self, ctxt: &Context) -> usize {
        match self.0 {
            BlockType::Empty => 0,
            BlockType::ShorthandFunc(_) => 1,
            BlockType::FunctionSig(func_idx) => {
                ctxt.module.function_types[func_idx as usize].num_results()
            }
        }
    }

    fn block_inputs(&self, ctxt: &Context) -> Box<dyn Iterator<Item = ValType>> {
        match self.0 {
            BlockType::Empty | BlockType::ShorthandFunc(_) => Box::new(std::iter::empty()),
            BlockType::FunctionSig(func_idx) => {
                Box::new(ctxt.module.function_types[func_idx as usize].params_iter())
            }
        }
    }

    fn block_inputs_count(&self, ctxt: &Context) -> usize {
        match self.0 {
            BlockType::Empty | BlockType::ShorthandFunc(_) => 0,
            BlockType::FunctionSig(func_idx) => {
                ctxt.module.function_types[func_idx as usize].num_params()
            }
        }
    }
}

pub(crate) fn validate_and_extract_result_from_stack(
    ctxt: &mut Context,
    out_params: &ResType,
    check_empty_stack: bool,
) -> Vec<VariableID> {
    let mut return_vars = Vec::new();
    let stack_depth = ctxt.stack.len();
    if stack_depth < out_params.len() {
        return ctxt.poison(ValidationError::Msg(
            "stack underflow in target label".to_string(),
        ));
    }
    if check_empty_stack && stack_depth > out_params.len() {
        return ctxt.poison(ValidationError::Msg(format!(
            "unexpected stack state at end of function: {:?}, expected {:?}",
            ctxt.stack.stack[ctxt.stack.stack.len() - stack_depth..]
                .iter()
                .map(|var| var.type_)
                .collect::<Vec<_>>(),
            out_params
        )));
    }
    for (i, return_value) in out_params.iter().enumerate() {
        let idx = stack_depth - out_params.len() + i;
        let stack_var = &ctxt.stack[idx];
        if stack_var.type_ != *return_value {
            return ctxt.poison(ValidationError::Msg(
                "mismatched return type in target label".to_string(),
            ));
        }
        return_vars.push(stack_var.id);
    }
    return_vars
}

fn parse_until_next_end(
    i: &mut WasmStreamReader,
    ctxt: &mut Context,
    labels: &mut Vec<Label>,
    builder: &mut FunctionBuilder,
) -> ParseResult {
    let mut saved_stack = ParserStack::new();
    let saved_poison = ctxt.poison.take();
    let mut trash_builder = FunctionBuilder::new();
    trash_builder.start_bb();
    std::mem::swap(&mut saved_stack, &mut ctxt.stack);
    parse_basic_blocks(i, ctxt, labels, &mut trash_builder)?;
    ctxt.poison = saved_poison;
    ctxt.stack = saved_stack;

    // we can forget parsed basic blocks IFF we didn't parse an else-tag
    if let Some(last_parsed_bb) = trash_builder.current_bb_try_get() {
        if let BasicBlockGlue::ElseMarker { .. } = last_parsed_bb.terminator.clone() {
            builder.start_bb();
            // we can't use the parsed out_vars as we discard all parsed code => out_vars would be invalid
            builder.terminate_else(Vec::new());
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub(crate) struct Label {
    // id of the basic block jump target
    pub(crate) bb_id: BasicBlockID,
    pub(crate) result_type: ResType,

    // currently only required for loops
    pub(crate) loop_after_bb_id: Option<BasicBlockID>,
    pub(crate) loop_after_result_type: Option<ResType>,
}

fn parse_terminator(
    i: &mut WasmStreamReader,
    ctxt: &mut Context,
    labels: &mut Vec<Label>,
    builder: &mut FunctionBuilder,
) -> Result<(), ParserError> {
    match builder.current_bb_terminator().clone() {
        ControlInstruction::Block(block_type) => {
            let block_type = BTWrapper(block_type);
            let block_input_vars = block_type.setup_block_stack(ctxt);

            // complete leading bb
            let first_nested_block_id = BasicBlock::next_id();
            builder.terminate_jmp(first_nested_block_id, block_input_vars);

            // save label stack size outside of block
            let label_depth = labels.len();

            // add next label to jump to (one more recursion level)
            let after_block_bb_id = BasicBlock::next_id();
            let block_label = Label {
                bb_id: after_block_bb_id,
                result_type: block_type.block_returns(ctxt).collect(),
                loop_after_bb_id: None,
                loop_after_result_type: None,
            };
            labels.push(block_label.clone());

            builder.reserve_bb_with_phis(after_block_bb_id, ctxt, block_type.block_returns(ctxt));

            // parse block instructions until the block's "end"
            builder.start_bb_with_id(first_nested_block_id);
            parse_basic_blocks(i, ctxt, labels, builder)?;

            // restore outer scope
            labels.truncate(label_depth);
            ctxt.stack.unstash();

            // put phis onto stack for block tail / bbs after block
            builder.continue_bb(after_block_bb_id);
            builder.put_phi_inputs_on_stack(ctxt);

            // collect all other blocks until the next outside "end"
            parse_basic_blocks(i, ctxt, labels, builder)?;
        }

        ControlInstruction::Loop(block_type) => {
            let block_type = BTWrapper(block_type);
            let block_input_vars = block_type.setup_block_stack(ctxt);
            let leading_bb_id = builder.current_bb_get().id;

            let loop_hdr_bb_id = BasicBlock::next_id();
            let loop_body_bb_id = BasicBlock::next_id();
            let loop_exit_bb_id = BasicBlock::next_id();

            // loop entry
            {
                builder.reserve_bb_with_phis(loop_hdr_bb_id, ctxt, block_type.block_inputs(ctxt));
                builder.replace_phi_inputs_on_stack(ctxt);
                builder.terminate_jmp(
                    loop_body_bb_id,
                    builder
                        .current_bb_get()
                        .inputs
                        .iter()
                        .map(|phi| phi.out)
                        .collect(),
                );
            }

            // loop exit
            builder.reserve_bb_with_phis(loop_exit_bb_id, ctxt, block_type.block_returns(ctxt));

            // complete leading bb
            builder.continue_bb(leading_bb_id);
            builder.terminate_jmp(loop_hdr_bb_id, block_input_vars);

            // save label stack size outside of block
            let label_depth = labels.len();

            // add next label to jump to (one more recursion level)
            let block_label = Label {
                bb_id: loop_hdr_bb_id,
                result_type: block_type.block_inputs(ctxt).collect(),
                loop_after_bb_id: Some(loop_exit_bb_id),
                loop_after_result_type: Some(block_type.block_returns(ctxt).collect()),
            };
            labels.push(block_label.clone());

            // parse block instructions until the block's "end"
            builder.start_bb_with_id(loop_body_bb_id);
            parse_basic_blocks(i, ctxt, labels, builder)?;

            // restore outer scope
            labels.truncate(label_depth);
            ctxt.stack.unstash();

            // collect all other blocks until the next outside "end"
            builder.continue_bb(loop_exit_bb_id);
            builder.put_phi_inputs_on_stack(ctxt);
            parse_basic_blocks(i, ctxt, labels, builder)?;
        }

        ControlInstruction::IfElse(block_type) => {
            let block_type = BTWrapper(block_type);
            let pred_bb_id = builder.current_bb_get().id;
            let cond_var = ctxt.pop_var_with_type(&ValType::i32()).id;

            let if_else_exit_bb = BasicBlock::next_id();
            builder.reserve_bb_with_phis(if_else_exit_bb, ctxt, block_type.block_returns(ctxt));

            // save label stack size outside of block
            let label_depth = labels.len();

            // add next label to jump to (one more recursion level)
            let block_label = Label {
                bb_id: if_else_exit_bb,
                loop_after_bb_id: None,
                loop_after_result_type: None,
                result_type: block_type.block_returns(ctxt).collect(),
            };
            labels.push(block_label.clone());

            let block_input_vars = block_type.setup_block_stack(ctxt);

            let target_if_true = BasicBlock::next_id();
            builder.start_bb_with_id(target_if_true);
            parse_basic_blocks(i, ctxt, labels, builder)?;

            if let BasicBlockGlue::ElseMarker {
                output_vars: ref end_of_then_out_vars,
            } = builder.current_bb_get().terminator
            {
                if end_of_then_out_vars.len() != block_type.block_returns_count(ctxt) {
                    // this is only a marker block and it only means trouble keeping it
                    builder.eliminate_current_bb();
                } else {
                    // if-else-end -> overwrite elsemarker (= end of "then" block) with direct jmp to after if-else
                    builder.terminate_jmp(if_else_exit_bb, end_of_then_out_vars.clone());
                }
                // restore state from if-statement
                ctxt.stack.unstash();
                ctxt.stack.stash();
                for (var, type_) in block_input_vars.iter().zip(block_type.block_inputs(ctxt)) {
                    ctxt.push_var(Variable { id: *var, type_ });
                }

                labels.truncate(label_depth);
                labels.push(block_label.clone());

                // parse "else" branch
                let target_if_false = BasicBlock::next_id();
                builder.start_bb_with_id(target_if_false);
                parse_basic_blocks(i, ctxt, labels, builder)?;

                builder.continue_bb(pred_bb_id);
                builder.terminate_jmp_cond(
                    cond_var,
                    target_if_true,
                    target_if_false,
                    block_input_vars,
                );
            } else {
                // if-end (no else)
                debug_assert!(
                    matches!(
                        builder.current_bb_get().terminator,
                        BasicBlockGlue::Unreachable
                            | BasicBlockGlue::Jmp { .. }
                            | BasicBlockGlue::JmpTable { .. }
                            | BasicBlockGlue::Return { .. }
                    ),
                    "{:?}",
                    builder.current_bb_get().terminator
                );
                builder.continue_bb(pred_bb_id);
                builder.terminate_jmp_cond(
                    cond_var,
                    target_if_true,
                    if_else_exit_bb,
                    block_input_vars,
                );
            }

            // restore outer scope
            ctxt.stack.unstash();
            labels.truncate(label_depth);

            // parse blocks after if-else
            builder.continue_bb(if_else_exit_bb);
            builder.put_phi_inputs_on_stack(ctxt);
            parse_basic_blocks(i, ctxt, labels, builder)?;
        }

        ControlInstruction::Br(label_idx) => {
            if labels.is_empty() || label_idx >= labels.len() as u32 {
                return Err(ParserError::Msg("label index out of bounds".to_string()));
            }
            let target_label = labels[labels.len() - label_idx as usize - 1].clone();
            let output_vars =
                validate_and_extract_result_from_stack(ctxt, &target_label.result_type, false);
            builder.terminate_jmp(target_label.bb_id, output_vars);

            // unconditional branch -> following blocks only need to parsed, but not validated
            parse_until_next_end(i, ctxt, labels, builder)?;
        }

        ControlInstruction::BrIf(label_idx) => {
            let cond_var = ctxt.pop_var_with_type(&ValType::i32()).id;
            let target_if_false = BasicBlock::next_id();
            let target_if_true = labels[labels.len() - label_idx as usize - 1].clone();
            let output_vars =
                validate_and_extract_result_from_stack(ctxt, &target_if_true.result_type, false);

            builder.terminate_jmp_cond(
                cond_var,
                target_if_true.bb_id,
                target_if_false,
                output_vars,
            );
            builder.start_bb_with_id(target_if_false);
            parse_basic_blocks(i, ctxt, labels, builder)?;
        }

        ControlInstruction::BrTable(default_label, label_table) => {
            let selector_var = ctxt.pop_var_with_type(&ValType::i32()).id;
            let default_bb = if default_label >= labels.len() as u32 {
                return Err(ParserError::Msg("label index out of bounds".to_string()));
            } else {
                labels[labels.len() - default_label as usize - 1].clone()
            };
            let default_output_vars =
                validate_and_extract_result_from_stack(ctxt, &default_bb.result_type, false);

            let (targets, targets_output_vars): (Vec<BasicBlockID>, Vec<Vec<VariableID>>) =
                label_table
                    .into_iter()
                    .map(|label_idx| &labels[labels.len() - label_idx as usize - 1])
                    .map(|target_label| {
                        let output_vars: Vec<VariableID> = validate_and_extract_result_from_stack(
                            ctxt,
                            &target_label.result_type,
                            false,
                        );
                        (target_label.bb_id, output_vars)
                    })
                    .unzip();
            builder.terminate_jmp_table(
                selector_var,
                targets,
                targets_output_vars,
                default_bb.bb_id,
                default_output_vars,
            );

            // unconditional branch -> following blocks only need to parsed, but not validated
            parse_until_next_end(i, ctxt, labels, builder)?;
        }

        ControlInstruction::Unreachable => {
            builder.terminate_unreachable();
            // parse away following junk
            parse_until_next_end(i, ctxt, labels, builder)?;
        }

        ControlInstruction::Call(func_idx) => {
            if func_idx > ctxt.module.ir.functions.len() as u32 {
                ctxt.poison(ValidationError::Msg(
                    "call function index out of bounds".to_string(),
                ))
            }
            let func_type = ctxt
                .module
                .ir
                .functions
                .get(func_idx as usize)
                .and_then(|func| {
                    ctxt.module
                        .function_types
                        .get(func.type_idx as usize)
                        .cloned()
                })
                .unwrap();
            let call_params =
                validate_and_extract_result_from_stack(ctxt, &func_type.params(), false);
            // pop all parameters from the stack
            ctxt.stack
                .stack
                .truncate(ctxt.stack.stack.len() - call_params.len());

            let return_vars = func_type
                .results_iter()
                .map(|val| {
                    let var = ctxt.create_var(val);
                    let tmp = var.id;
                    ctxt.push_var(var);
                    tmp
                })
                .collect();
            let return_bb = BasicBlock::next_id();
            builder.terminate_call(func_idx, return_bb, call_params, return_vars);

            // parse continuation basic blocks
            builder.start_bb_with_id(return_bb);
            parse_basic_blocks(i, ctxt, labels, builder)?;
        }

        ControlInstruction::CallIndirect(type_idx, table_idx) => {
            if table_idx > ctxt.module.tables.len() as u32 {
                ctxt.poison(ValidationError::Msg(
                    "icall table index out of bounds".to_string(),
                ))
            } else if !matches!(
                ctxt.module.tables[table_idx as usize].r#type.ref_type,
                RefType::FunctionReference
            ) {
                ctxt.poison(ValidationError::Msg(
                    "icall table type mismatch".to_string(),
                ))
            }

            if type_idx > ctxt.module.function_types.len() as u32 {
                ctxt.poison(ValidationError::Msg(
                    "icall function index out of bounds".to_string(),
                ))
            }

            let selector_var = ctxt.pop_var_with_type(&ValType::i32()).id;
            let func_type = ctxt.module.function_types.get(type_idx as usize).unwrap();
            let call_params =
                validate_and_extract_result_from_stack(ctxt, &func_type.params(), false);
            // pop all parameters from the stack
            ctxt.stack
                .stack
                .truncate(ctxt.stack.stack.len() - call_params.len());

            let return_vars = func_type
                .results_iter()
                .map(|val| {
                    let var = ctxt.create_var(val);
                    let tmp = var.id;
                    ctxt.push_var(var);
                    tmp
                })
                .collect();
            let return_bb = BasicBlock::next_id();
            builder.terminate_call_indirect(
                type_idx,
                selector_var,
                table_idx,
                return_bb,
                call_params,
                return_vars,
            );

            // parse continuation basic blocks
            builder.start_bb_with_id(return_bb);
            parse_basic_blocks(i, ctxt, labels, builder)?;
        }

        ControlInstruction::End => {
            match labels.len() {
                1 => {
                    // return from function
                    let func_scope_label = labels.last().unwrap();
                    let return_vars = validate_and_extract_result_from_stack(
                        ctxt,
                        &func_scope_label.result_type,
                        true,
                    );
                    builder.terminate_return(return_vars);
                }
                0 => {
                    // parsing after return from function scope OR parsing outside of function => unreachable
                    builder.terminate_unreachable();
                }
                _ => {
                    // return from block, jump to last label
                    let last_label = labels.last().unwrap();

                    let output_vars = validate_and_extract_result_from_stack(
                        ctxt,
                        last_label
                            .loop_after_result_type
                            .as_ref()
                            .unwrap_or(&last_label.result_type),
                        true,
                    );
                    builder.terminate_jmp(
                        last_label.loop_after_bb_id.unwrap_or(last_label.bb_id),
                        output_vars,
                    );
                }
            }
        }

        ControlInstruction::Return => {
            // validate return parameter types
            let func_scope_label = labels.first().ok_or(ParserError::Msg(
                "return instruction outside of function scope".to_string(),
            ))?;
            let return_vars =
                validate_and_extract_result_from_stack(ctxt, &func_scope_label.result_type, false);
            builder.terminate_return(return_vars);
            // parse away the following end instruction and any junk that might follow
            parse_until_next_end(i, ctxt, labels, builder)?;
        }

        ControlInstruction::Else => {
            let ifscopelabel = labels.last().unwrap();
            let output_vars =
                validate_and_extract_result_from_stack(ctxt, &ifscopelabel.result_type, false);
            builder.terminate_else(output_vars);
            // stop parsing here, because the "else" block is parsed in the "if" block
        }

        ControlInstruction::Nop => {
            // we don't parse this at all.
            unreachable!()
        }
    }
    Ok(())
}

pub(crate) fn parse_basic_blocks(
    i: &mut WasmStreamReader,
    ctxt: &mut Context,
    labels: &mut Vec<Label>,
    builder: &mut FunctionBuilder,
) -> Result<(), ParserError> {
    let instrs = builder.current_bb_instrs();
    while !instrs.is_finished() {
        let opcode: u8 = i.read_byte()?;
        LVL1_JMP_TABLE[opcode as usize](ctxt, i, instrs)?;
    }
    parse_terminator(i, ctxt, labels, builder)
}
