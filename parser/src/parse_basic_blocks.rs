use crate::{
    error::ParserError, instructions::meta::new_phinode, wasm_stream_reader::WasmStreamReader,
};
use ir::{
    basic_block::{BasicBlock, BasicBlockGlue, BasicBlockID, BasicBlockStorage},
    instructions::{
        meta::{self},
        Instruction, VariableID,
    },
    structs::{instruction::ControlInstruction, table::Tablelike},
    InstructionEncoder,
};
use wasm_types::instruction::BlockType;
use wasm_types::{NumType, RefType, ResType, ValType};

use super::{opcode_tbl::LVL1_JMP_TABLE, Context, ParseResult, ParserStack, ValidationError};

fn setup_block_stack(block_type: BlockType, ctxt: &mut Context) {
    let input_signature = get_block_input_signature(ctxt, block_type);
    let input_length = input_signature.len();
    // divide stack into block stack (input params) and remaining stack (output params)
    ctxt.stack.stash_with_keep(input_length);
    for (i, input_var) in input_signature.iter().enumerate() {
        if ctxt.stack[i].type_ != *input_var {
            ctxt.poison(ValidationError::Msg(
                "mismatched input signature in target label".to_string(),
            ))
        }
    }
}

fn get_block_return_signature(ctxt: &Context, block_type: BlockType) -> ResType {
    match block_type {
        BlockType::Empty => Vec::new(),
        BlockType::ShorthandFunc(val_type) => vec![val_type],
        BlockType::FunctionSig(func_idx) => ctxt.module.function_types[func_idx as usize].1.clone(),
    }
}

fn get_block_input_signature(ctxt: &Context, block_type: BlockType) -> ResType {
    match block_type {
        BlockType::Empty | BlockType::ShorthandFunc(_) => Vec::new(),
        BlockType::FunctionSig(func_idx) => ctxt.module.function_types[func_idx as usize].0.clone(),
    }
}

fn validate_and_extract_result_from_stack(
    ctxt: &mut Context,
    out_params: &ResType,
) -> Vec<VariableID> {
    let mut return_vars = Vec::new();
    let stack_depth = ctxt.stack.len();
    if stack_depth < out_params.len() {
        return ctxt.poison(ValidationError::Msg(
            "stack underflow in target label".to_string(),
        ));
    }
    for (i, return_value) in out_params.iter().rev().enumerate() {
        let idx = stack_depth - i - 1;
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
    bbs: &mut Vec<BasicBlock>,
) -> ParseResult {
    let mut saved_stack = ParserStack::new();
    std::mem::swap(&mut saved_stack, &mut ctxt.stack);
    let parsed_blocks = parse_basic_blocks(i, ctxt, labels, BasicBlock::next_id(), None)?;
    ctxt.poison = None;
    ctxt.stack = saved_stack;

    // we can forget parsed basic blocks IFF we didn't parse an else-tag
    if let Some(last_parsed_bb) = parsed_blocks.last() {
        if let BasicBlockGlue::ElseMarker { output_vars } = last_parsed_bb.terminator.clone() {
            let mut empty_bb = BasicBlock::new(BasicBlock::next_id());
            empty_bb.terminator = BasicBlockGlue::ElseMarker { output_vars };
            bbs.push(empty_bb);
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub(crate) struct Label {
    pub(crate) bb_id: BasicBlockID,
    pub(crate) result_type: ResType,
}

fn parse_terminator(
    i: &mut WasmStreamReader,
    instruction_storage: BasicBlockStorage,
    ctxt: &mut Context,
    labels: &mut Vec<Label>,
    start_id: u32,
) -> Result<Vec<BasicBlock>, ParserError> {
    let mut bbs = Vec::new();
    match instruction_storage.terminator.clone() {
        ControlInstruction::Block(block_type) | ControlInstruction::Loop(block_type) => {
            let is_loop = matches!(instruction_storage.terminator, ControlInstruction::Loop(_));
            setup_block_stack(block_type.clone(), ctxt);

            let after_block_instr_bb_id = BasicBlock::next_id();
            let first_nested_block_id = BasicBlock::next_id();
            let jump_target_bb_id = if is_loop {
                first_nested_block_id
            } else {
                after_block_instr_bb_id
            };

            // complete leading bb
            let mut pre_block_instr_bb = BasicBlock::new(start_id);
            pre_block_instr_bb.instructions = instruction_storage;
            pre_block_instr_bb.terminator = BasicBlockGlue::Jmp {
                target: first_nested_block_id,
                output_vars: Vec::new(),
            };
            bbs.push(pre_block_instr_bb);

            // save label stack size outside of block
            let label_depth = labels.len();

            // add next label to jump to (one more recursion level)
            let block_label = Label {
                bb_id: jump_target_bb_id,
                result_type: if !is_loop {
                    get_block_return_signature(ctxt, block_type.clone())
                } else {
                    Vec::new()
                },
            };
            labels.push(block_label.clone());

            // parse block instructions until the block's "end"
            // TODO: stack should be copied here, so that the block can't alter the stack for the outer block
            let mut nested_blocks =
                parse_basic_blocks(i, ctxt, labels, first_nested_block_id, None)?;
            bbs.append(&mut nested_blocks);

            // restore outer scope
            labels.truncate(label_depth);
            ctxt.stack.unstash();

            // parse all jumps outside of the block to the block lable as phis (TODO: optimize this linear search through a predecessor list)
            let mut phis = get_block_return_signature(ctxt, block_type.clone())
                .into_iter()
                .map(|val| new_phinode(Vec::new(), val, ctxt))
                .collect::<Vec<_>>();
            for parsed_block in nested_blocks.iter() {
                if parsed_block
                    .successors()
                    .any(|s| s == after_block_instr_bb_id)
                {
                    for (phi, bb_out_var) in phis
                        .iter_mut()
                        .zip(parsed_block.target_out_vars(after_block_instr_bb_id))
                    {
                        phi.inputs.push((parsed_block.id, bb_out_var));
                    }
                }
            }

            let mut after_block_instrs = InstructionEncoder::new();
            for phi in phis.into_iter() {
                meta::PhiNode::serialize(phi, &mut after_block_instrs);
            }

            // collect all other blocks until the next outside "end"
            let mut after_blocks = parse_basic_blocks(
                i,
                ctxt,
                labels,
                after_block_instr_bb_id,
                Some(after_block_instrs),
            )?;
            bbs.append(&mut after_blocks);
            Ok(bbs)
        }

        ControlInstruction::IfElse(block_type) => {
            let mut pred_bb: BasicBlock = BasicBlock::new(start_id);
            pred_bb.instructions = instruction_storage;
            let cond_var = ctxt.pop_var_with_type(&ValType::Number(NumType::I32)).id;

            let bb_after_ifelse = BasicBlock::next_id();

            // save label stack size outside of block
            let label_depth = labels.len();

            // add next label to jump to (one more recursion level)
            let block_label = Label {
                bb_id: bb_after_ifelse,
                result_type: get_block_return_signature(ctxt, block_type.clone()),
            };
            labels.push(block_label.clone());

            let input_signature = get_block_input_signature(ctxt, block_type.clone());
            let input_length = input_signature.len();
            let mut input_vars = Vec::new();
            for (idx, val) in input_signature.into_iter().enumerate() {
                let opt = ctxt.stack.get(ctxt.stack.len() - 1 - idx).cloned();
                match opt {
                    None => ctxt.poison(ValidationError::Msg(
                        "stack underflow in target label".to_string(),
                    )),
                    Some(var) if var.type_ != val => ctxt.poison(ValidationError::Msg(
                        "mismatched input signature in target label".to_string(),
                    )),
                    Some(var) => input_vars.push(var),
                }
            }
            ctxt.stack.stash_with_keep(input_length);

            let target_if_true = BasicBlock::next_id();
            let mut blocks_for_true_path =
                parse_basic_blocks(i, ctxt, labels, target_if_true, None)?;
            if let BasicBlockGlue::ElseMarker {
                output_vars: ref end_of_then_out_vars,
            } = blocks_for_true_path.last().unwrap().terminator
            {
                // if-else-end
                blocks_for_true_path.last_mut().unwrap().terminator = BasicBlockGlue::Jmp {
                    target: bb_after_ifelse,
                    output_vars: end_of_then_out_vars.clone(),
                };
                // restore state from if-statement
                ctxt.stack.unstash();
                ctxt.stack.stash();
                for var in input_vars.into_iter().rev() {
                    ctxt.push_var(var);
                }

                labels.truncate(label_depth);
                labels.push(block_label.clone());

                // parse "else" branch
                let target_if_false = BasicBlock::next_id();
                let mut blocks_for_false_path =
                    parse_basic_blocks(i, ctxt, labels, target_if_false, None)?;

                pred_bb.terminator = BasicBlockGlue::JmpCond {
                    cond_var,
                    target_if_true,
                    target_if_false,
                    output_vars: Vec::new(),
                };
                bbs.push(pred_bb);
                bbs.append(&mut blocks_for_true_path);
                bbs.append(&mut blocks_for_false_path);
            } else {
                // if-end (no else)
                debug_assert!(
                    matches!(
                        blocks_for_true_path.last().unwrap().terminator,
                        BasicBlockGlue::Unreachable
                            | BasicBlockGlue::Jmp { .. }
                            | BasicBlockGlue::JmpTable { .. }
                            | BasicBlockGlue::Return { .. }
                    ),
                    "{:?}",
                    blocks_for_true_path.last().unwrap().terminator
                );
                pred_bb.terminator = BasicBlockGlue::JmpCond {
                    cond_var,
                    target_if_true,
                    target_if_false: bb_after_ifelse,
                    // empty output vars, because this is a jump "inside" of a block and not outside
                    output_vars: Vec::new(),
                };
                bbs.push(pred_bb);
                bbs.append(&mut blocks_for_true_path);
            }

            // restore outer scope
            ctxt.stack.unstash();
            labels.truncate(label_depth);

            // parse all jumps outside of the block to the block lable as phis (TODO: optimize this linear search through a predecessor list)
            let mut phis = get_block_return_signature(ctxt, block_type.clone())
                .into_iter()
                .map(|val| new_phinode(Vec::new(), val, ctxt))
                .collect::<Vec<_>>();
            for parsed_block in bbs.iter() {
                if parsed_block.successors().any(|s| s == bb_after_ifelse) {
                    for (phi, bb_out_var) in phis
                        .iter_mut()
                        .zip(parsed_block.target_out_vars(bb_after_ifelse))
                    {
                        phi.inputs.push((parsed_block.id, bb_out_var));
                    }
                }
            }

            let mut blocks_after_ifelse =
                parse_basic_blocks(i, ctxt, labels, bb_after_ifelse, None)?;
            bbs.append(&mut blocks_after_ifelse);
            Ok(bbs)
        }

        ControlInstruction::Br(label_idx) => {
            if labels.is_empty() || label_idx >= labels.len() as u32 {
                return Err(ParserError::Msg("label index out of bounds".to_string()));
            }

            let mut bb: BasicBlock = BasicBlock::new(start_id);
            bb.instructions = instruction_storage;
            let target_label = labels[labels.len() - label_idx as usize - 1].clone();
            let output_vars =
                validate_and_extract_result_from_stack(ctxt, &target_label.result_type);

            bb.terminator = BasicBlockGlue::Jmp {
                target: target_label.bb_id,
                output_vars,
            };
            bbs.push(bb);

            // unconditional branch -> following blocks only need to parsed, but not validated
            parse_until_next_end(i, ctxt, labels, &mut bbs)?;

            Ok(bbs)
        }

        ControlInstruction::BrIf(label_idx) => {
            let mut bb: BasicBlock = BasicBlock::new(start_id);
            bb.instructions = instruction_storage;
            let cond_var = ctxt.pop_var_with_type(&ValType::Number(NumType::I32)).id;
            let cont_bb_id = BasicBlock::next_id();
            let target_if_true = labels[labels.len() - label_idx as usize - 1].clone();
            let output_vars =
                validate_and_extract_result_from_stack(ctxt, &target_if_true.result_type);

            bb.terminator = BasicBlockGlue::JmpCond {
                cond_var,
                target_if_true: target_if_true.bb_id,
                target_if_false: cont_bb_id,
                output_vars,
            };
            bbs.push(bb);

            let mut cont_bbs = parse_basic_blocks(i, ctxt, labels, cont_bb_id, None)?;
            bbs.append(&mut cont_bbs);
            Ok(bbs)
        }

        ControlInstruction::BrTable(default_label, label_table) => {
            let mut bb: BasicBlock = BasicBlock::new(start_id);
            bb.instructions = instruction_storage;
            let cond_var = ctxt.pop_var_with_type(&ValType::Number(NumType::I32)).id;

            let default_bb = if default_label >= labels.len() as u32 {
                return Err(ParserError::Msg("label index out of bounds".to_string()));
            } else {
                labels[labels.len() - default_label as usize - 1].clone()
            };
            let default_output_vars =
                validate_and_extract_result_from_stack(ctxt, &default_bb.result_type);

            let (target_bbs, target_bbs_out_vars): (Vec<BasicBlockID>, Vec<Vec<VariableID>>) =
                label_table
                    .into_iter()
                    .map(|label_idx| &labels[labels.len() - label_idx as usize - 1])
                    .map(|target_label| {
                        let output_vars: Vec<VariableID> =
                            validate_and_extract_result_from_stack(ctxt, &target_label.result_type);
                        (target_label.bb_id, output_vars)
                    })
                    .unzip();
            bb.terminator = BasicBlockGlue::JmpTable {
                cond_var,
                targets: target_bbs,
                targets_output_vars: target_bbs_out_vars,
                default_target: default_bb.bb_id,
                default_output_vars,
            };
            bbs.push(bb);

            // unconditional branch -> following blocks only need to parsed, but not validated
            parse_until_next_end(i, ctxt, labels, &mut bbs)?;

            Ok(bbs)
        }

        ControlInstruction::Unreachable => {
            let mut bb: BasicBlock = BasicBlock::new(start_id);
            bb.instructions = instruction_storage;
            bb.terminator = BasicBlockGlue::Unreachable;
            bbs.push(bb);

            // parse away following junk
            parse_until_next_end(i, ctxt, labels, &mut bbs)?;
            Ok(bbs)
        }

        ControlInstruction::End => {
            let mut bb: BasicBlock = BasicBlock::new(start_id);
            bb.instructions = instruction_storage;

            match labels.len() {
                1 => {
                    // return from function
                    let func_scope_label = labels.last().unwrap();
                    let return_vars =
                        validate_and_extract_result_from_stack(ctxt, &func_scope_label.result_type);
                    bb.terminator = BasicBlockGlue::Return { return_vars };
                }
                0 => {
                    // parsing after return from function scope OR parsing outside of function => unreachable
                    bb.terminator = BasicBlockGlue::Unreachable;
                }
                _ => {
                    // return from block, jump to last label
                    let last_label = labels.last().unwrap();
                    let output_vars =
                        validate_and_extract_result_from_stack(ctxt, &last_label.result_type);
                    bb.terminator = BasicBlockGlue::Jmp {
                        target: last_label.bb_id,
                        output_vars,
                    }
                }
            }
            bbs.push(bb);
            Ok(bbs)
        }

        ControlInstruction::Return => {
            // validate return parameter types
            let func_scope_label = labels.first().ok_or(ParserError::Msg(
                "return instruction outside of function scope".to_string(),
            ))?;
            let return_vars =
                validate_and_extract_result_from_stack(ctxt, &func_scope_label.result_type);

            let mut bb: BasicBlock = BasicBlock::new(start_id);
            bb.instructions = instruction_storage;
            bb.terminator = BasicBlockGlue::Return { return_vars };
            bbs.push(bb);

            // parse away the following end instruction and any junk that might follow
            parse_until_next_end(i, ctxt, labels, &mut bbs)?;
            Ok(bbs)
        }

        ControlInstruction::Call(func_idx) => {
            if func_idx > ctxt.module.ir.functions.len() as u32 {
                ctxt.poison(ValidationError::Msg(
                    "call function index out of bounds".to_string(),
                ))
            }

            let mut bb: BasicBlock = BasicBlock::new(start_id);
            bb.instructions = instruction_storage;

            let cont_bb_id = BasicBlock::next_id();
            let func = ctxt
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
            let call_params = validate_and_extract_result_from_stack(ctxt, &func.0);
            // pop all parameters from the stack
            ctxt.stack
                .stack
                .truncate(ctxt.stack.stack.len() - call_params.len());
            bb.terminator = BasicBlockGlue::Call {
                func_idx,
                return_bb: cont_bb_id,
                call_params,
            };
            bbs.push(bb);

            for return_var in func.1 {
                let var = ctxt.create_var(return_var);
                ctxt.push_var(var);
            }

            let mut cont_bbs = parse_basic_blocks(i, ctxt, labels, cont_bb_id, None)?;
            bbs.append(&mut cont_bbs);
            Ok(bbs)
        }

        ControlInstruction::CallIndirect(type_idx, table_idx) => {
            if table_idx > ctxt.module.tables.len() as u32 {
                ctxt.poison(ValidationError::Msg(
                    "icall table index out of bounds".to_string(),
                ))
            } else if !matches!(
                ctxt.module.tables[table_idx as usize].get_ref_type(),
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

            let mut bb: BasicBlock = BasicBlock::new(start_id);
            bb.instructions = instruction_storage;

            let cont_bb_id = BasicBlock::next_id();
            let selector_var = ctxt.pop_var_with_type(&ValType::Number(NumType::I32)).id;
            let func_type = ctxt.module.function_types.get(type_idx as usize).unwrap();
            let call_params = validate_and_extract_result_from_stack(ctxt, &func_type.0);
            // pop all parameters from the stack
            ctxt.stack
                .stack
                .truncate(ctxt.stack.stack.len() - call_params.len());
            bb.terminator = BasicBlockGlue::CallIndirect {
                type_idx,
                table_idx,
                selector_var,
                return_bb: cont_bb_id,
                call_params,
            };
            bbs.push(bb);

            for return_var in &func_type.1 {
                let var = ctxt.create_var(*return_var);
                ctxt.push_var(var);
            }

            let mut cont_bbs = parse_basic_blocks(i, ctxt, labels, cont_bb_id, None)?;
            bbs.append(&mut cont_bbs);
            Ok(bbs)
        }

        ControlInstruction::Else => {
            let mut bb: BasicBlock = BasicBlock::new(start_id);
            bb.instructions = instruction_storage;

            let ifscopelabel = labels.last().unwrap();
            let output_vars =
                validate_and_extract_result_from_stack(ctxt, &ifscopelabel.result_type);
            bb.terminator = BasicBlockGlue::ElseMarker { output_vars };
            bbs.push(bb);

            // stop parsing here, because the "else" block is parsed in the "if" block
            Ok(bbs)
        }

        ControlInstruction::Nop => {
            // we don't parse this at all.
            unreachable!()
        }
    }
}

pub(crate) fn parse_basic_blocks(
    i: &mut WasmStreamReader,
    ctxt: &mut Context,
    labels: &mut Vec<Label>,
    start_id: u32,
    instruction_writer: Option<InstructionEncoder>,
) -> Result<Vec<BasicBlock>, ParserError> {
    let mut instruction_writer = instruction_writer.unwrap_or_default();
    while !instruction_writer.is_finished() {
        let opcode: u8 = i.read_byte()?;
        LVL1_JMP_TABLE[opcode as usize](ctxt, i, &mut instruction_writer)?;
    }

    let instruction_storage = instruction_writer.extract_data();
    parse_terminator(i, instruction_storage, ctxt, labels, start_id)
}
