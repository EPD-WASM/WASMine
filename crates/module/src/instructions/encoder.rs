use super::{basic_block::BasicBlockStorage, *};
use crate::{objects::instruction::ControlInstruction, utils::integer_traits::Integer};
use instruction_consumer::InstructionConsumer;
use std::collections::VecDeque;
use wasm_types::{InstructionType, NumericInstructionCategory, ValType};

#[derive(Clone)]
pub struct InstructionEncoder {
    storage: BasicBlockStorage,
    finished: bool,
}

impl Default for InstructionEncoder {
    fn default() -> Self {
        InstructionEncoder {
            storage: BasicBlockStorage {
                immediate_storage: VecDeque::with_capacity(100),
                variable_storage: VecDeque::with_capacity(20),
                type_storage: VecDeque::with_capacity(5),
                instruction_storage: VecDeque::with_capacity(10),
                terminator: ControlInstruction::Unreachable,
                inputs: Vec::new(),
            },
            finished: false,
        }
    }
}

impl InstructionEncoder {
    pub fn new() -> InstructionEncoder {
        Self::default()
    }

    pub fn extract_data(self) -> BasicBlockStorage {
        self.storage
    }

    fn write_instruction_type(&mut self, type_: InstructionType) {
        self.storage.instruction_storage.push_back(type_);
    }

    fn write_immediate<T: Integer>(&mut self, imm: T) {
        self.storage.immediate_storage.extend(imm.to_bytes());
    }

    fn write_variable(&mut self, var: VariableID) {
        self.storage.variable_storage.push_back(var);
    }

    fn write_value_type(&mut self, type_: ValType) {
        self.storage.type_storage.push_back(type_);
    }
}

impl InstructionConsumer for InstructionEncoder {
    fn is_finished(&self) -> bool {
        self.finished
    }

    fn write_ibinary(&mut self, i: IBinaryInstruction) {
        self.write_instruction_type(InstructionType::Numeric(
            NumericInstructionCategory::IBinary(i.op.clone()),
        ));
        self.write_value_type(ValType::Number(i.types));
        self.write_variable(i.lhs);
        self.write_variable(i.rhs);
        self.write_variable(i.out1);
    }

    fn write_fbinary(&mut self, i: FBinaryInstruction) {
        self.write_instruction_type(InstructionType::Numeric(
            NumericInstructionCategory::FBinary(i.op.clone()),
        ));
        self.write_value_type(ValType::Number(i.types));
        self.write_variable(i.lhs);
        self.write_variable(i.rhs);
        self.write_variable(i.out1);
    }

    fn write_iunary(&mut self, i: IUnaryInstruction) {
        self.write_instruction_type(InstructionType::Numeric(
            NumericInstructionCategory::IUnary(i.op.clone()),
        ));
        self.write_value_type(ValType::Number(i.types));
        self.write_variable(i.in1);
        self.write_variable(i.out1);
    }

    fn write_funary(&mut self, i: FUnaryInstruction) {
        self.write_instruction_type(InstructionType::Numeric(
            NumericInstructionCategory::FUnary(i.op.clone()),
        ));
        self.write_value_type(ValType::Number(i.types));
        self.write_variable(i.in1);
        self.write_variable(i.out1);
    }

    fn write_block(&mut self, i: Block) {
        self.finish(ControlInstruction::Block(i.block_type));
    }

    fn write_brif(&mut self, i: BrIf) {
        self.finish(ControlInstruction::BrIf(i.label_idx));
    }

    fn write_br(&mut self, i: Br) {
        self.finish(ControlInstruction::Br(i.label_idx));
    }

    fn write_br_table(&mut self, i: BrTable) {
        self.finish(ControlInstruction::BrTable(
            i.default_label_idx,
            i.label_indices,
        ));
    }

    fn write_call_indirect(&mut self, i: CallIndirect) {
        self.finish(ControlInstruction::CallIndirect(i.type_idx, i.table_idx));
    }

    fn write_call(&mut self, i: Call) {
        self.finish(ControlInstruction::Call(i.func_idx));
    }

    fn write_if_else(&mut self, i: IfElse) {
        self.finish(ControlInstruction::IfElse(i.block_type));
    }

    fn write_else(&mut self) {
        self.finish(ControlInstruction::Else);
    }

    fn write_loop(&mut self, i: Loop) {
        self.finish(ControlInstruction::Loop(i.block_type));
    }

    fn write_end(&mut self) {
        self.finish(ControlInstruction::End);
    }

    fn write_return(&mut self) {
        self.finish(ControlInstruction::Return);
    }

    fn write_unreachable(&mut self) {
        self.finish(ControlInstruction::Unreachable);
    }

    fn write_store(&mut self, i: StoreInstruction) {
        self.write_instruction_type(InstructionType::Memory(MemoryInstructionCategory::Store(
            i.operation.clone(),
        )));
        self.write_immediate(i.memarg.align);
        self.write_immediate(i.memarg.offset);
        self.write_variable(i.addr_in);
        self.write_variable(i.value_in);
        self.write_value_type(ValType::Number(i.in_type))
    }

    fn write_load(&mut self, i: LoadInstruction) {
        self.write_instruction_type(InstructionType::Memory(MemoryInstructionCategory::Load(
            i.operation.clone(),
        )));
        self.write_immediate(i.memarg.align);
        self.write_immediate(i.memarg.offset);
        self.write_variable(i.addr);
        self.write_variable(i.out1);
        self.write_value_type(ValType::Number(i.out1_type))
    }

    fn write_test(&mut self, i: ITestInstruction) {
        self.write_instruction_type(InstructionType::Numeric(NumericInstructionCategory::ITest(
            i.op.clone(),
        )));
        self.write_value_type(ValType::Number(i.input_type));
        self.write_variable(i.in1);
        self.write_variable(i.out1);
    }

    fn write_memory_size(&mut self, i: MemorySizeInstruction) {
        self.write_instruction_type(InstructionType::Memory(MemoryInstructionCategory::Memory(
            MemoryOp::Size,
        )));
        self.write_variable(i.out1)
    }

    fn write_memory_grow(&mut self, i: MemoryGrowInstruction) {
        self.write_instruction_type(InstructionType::Memory(MemoryInstructionCategory::Memory(
            MemoryOp::Grow,
        )));
        self.write_variable(i.in1);
        self.write_variable(i.out1);
    }

    fn write_memory_copy(&mut self, i: MemoryCopyInstruction) {
        self.write_instruction_type(InstructionType::Memory(MemoryInstructionCategory::Memory(
            MemoryOp::Copy,
        )));
        self.write_variable(i.n);
        self.write_variable(i.s);
        self.write_variable(i.d);
    }

    fn write_memory_fill(&mut self, i: MemoryFillInstruction) {
        self.write_instruction_type(InstructionType::Memory(MemoryInstructionCategory::Memory(
            MemoryOp::Fill,
        )));
        self.write_variable(i.n);
        self.write_variable(i.val);
        self.write_variable(i.d);
    }

    fn write_memory_init(&mut self, i: MemoryInitInstruction) {
        self.write_instruction_type(InstructionType::Memory(MemoryInstructionCategory::Memory(
            MemoryOp::Init,
        )));
        self.write_immediate(i.data_idx);
        self.write_variable(i.n);
        self.write_variable(i.s);
        self.write_variable(i.d);
    }

    fn write_data_drop(&mut self, i: DataDropInstruction) {
        self.write_instruction_type(InstructionType::Memory(MemoryInstructionCategory::Memory(
            MemoryOp::Drop,
        )));
        self.write_immediate(i.data_idx)
    }

    fn write_trunc(&mut self, i: TruncInstruction) {
        self.write_instruction_type(InstructionType::Numeric(
            NumericInstructionCategory::Conversion(ConversionOp::Trunc),
        ));
        self.write_variable(i.in1);
        self.write_value_type(ValType::Number(i.in1_type));

        self.write_variable(i.out1);
        self.write_value_type(ValType::Number(i.out1_type));

        self.write_immediate(i.signed as u8);
    }

    fn write_trunc_saturation(&mut self, i: TruncSaturationInstruction) {
        self.write_instruction_type(InstructionType::Numeric(
            NumericInstructionCategory::Conversion(ConversionOp::TruncSat),
        ));
        self.write_variable(i.in1);
        self.write_variable(i.out1);
        self.write_value_type(ValType::Number(i.in1_type));
        self.write_value_type(ValType::Number(i.out1_type));
        self.write_immediate(i.signed as u8);
    }

    fn write_irelational(&mut self, i: IRelationalInstruction) {
        self.write_instruction_type(InstructionType::Numeric(
            NumericInstructionCategory::IRelational(i.op.clone()),
        ));
        self.write_value_type(ValType::Number(i.input_types));
        self.write_variable(i.in1);
        self.write_variable(i.in2);
        self.write_variable(i.out1);
    }

    fn write_frelational(&mut self, i: FRelationalInstruction) {
        self.write_instruction_type(InstructionType::Numeric(
            NumericInstructionCategory::FRelational(i.op.clone()),
        ));
        self.write_value_type(ValType::Number(i.input_types));
        self.write_variable(i.in1);
        self.write_variable(i.in2);
        self.write_variable(i.out1);
    }

    fn write_wrap(&mut self, i: WrapInstruction) {
        self.write_instruction_type(InstructionType::Numeric(
            NumericInstructionCategory::Conversion(ConversionOp::Wrap),
        ));
        self.write_variable(i.in1);
        self.write_variable(i.out1);
    }

    fn write_convert(&mut self, i: ConvertInstruction) {
        self.write_instruction_type(InstructionType::Numeric(
            NumericInstructionCategory::Conversion(ConversionOp::Convert),
        ));
        self.write_variable(i.in1);
        self.write_value_type(ValType::Number(i.in1_type));

        self.write_variable(i.out1);
        self.write_value_type(ValType::Number(i.out1_type));

        self.write_immediate(i.signed as u8);
    }

    fn write_reinterpret(&mut self, i: ReinterpretInstruction) {
        self.write_instruction_type(InstructionType::Numeric(
            NumericInstructionCategory::Conversion(ConversionOp::Reinterpret),
        ));

        self.write_variable(i.in1);
        self.write_value_type(ValType::Number(i.in1_type));

        self.write_variable(i.out1);
        self.write_value_type(ValType::Number(i.out1_type));
    }

    fn write_extend_bits(&mut self, i: ExtendBitsInstruction) {
        self.write_instruction_type(InstructionType::Numeric(
            NumericInstructionCategory::Conversion(ConversionOp::ExtendBits),
        ));
        self.write_variable(i.in1);
        self.write_value_type(ValType::Number(i.in1_type));

        self.write_immediate(i.input_size);

        self.write_variable(i.out1);
        self.write_value_type(ValType::Number(i.out1_type));
    }

    fn write_extend_type(&mut self, i: ExtendTypeInstruction) {
        self.write_instruction_type(InstructionType::Numeric(
            NumericInstructionCategory::Conversion(ConversionOp::ExtendType),
        ));
        self.write_immediate(i.signed as u8);
        self.write_variable(i.in1);
        self.write_variable(i.out1);
    }

    fn write_demote(&mut self, i: DemoteInstruction) {
        self.write_instruction_type(InstructionType::Numeric(
            NumericInstructionCategory::Conversion(ConversionOp::Demote),
        ));
        self.write_variable(i.in1);
        self.write_variable(i.out1);
    }

    fn write_promote(&mut self, i: PromoteInstruction) {
        self.write_instruction_type(InstructionType::Numeric(
            NumericInstructionCategory::Conversion(ConversionOp::Promote),
        ));
        self.write_variable(i.in1);
        self.write_variable(i.out1);
    }

    fn write_const(&mut self, i: Constant) {
        self.write_instruction_type(InstructionType::Numeric(
            NumericInstructionCategory::Constant,
        ));
        self.write_immediate(i.imm.as_u64());
        self.write_variable(i.out1);
        self.write_value_type(ValType::Number(i.out1_type));
    }

    fn write_reference_is_null(&mut self, i: ReferenceIsNullInstruction) {
        self.write_instruction_type(InstructionType::Reference(
            ReferenceInstructionType::RefIsNull,
        ));
        self.write_variable(i.in1);
        self.write_value_type(i.in1_type);
        self.write_variable(i.out1);
    }

    fn write_reference_null(&mut self, i: ReferenceNullInstruction) {
        self.write_instruction_type(InstructionType::Reference(
            ReferenceInstructionType::RefNull,
        ));
        self.write_variable(i.out1);
        self.write_value_type(ValType::Reference(i.out1_type));
    }

    fn write_reference_function(&mut self, i: ReferenceFunctionInstruction) {
        self.write_instruction_type(InstructionType::Reference(
            ReferenceInstructionType::RefFunc,
        ));
        self.write_variable(i.out1);
        self.write_immediate(i.func_idx);
    }

    fn write_select(&mut self, i: SelectInstruction) {
        self.write_instruction_type(InstructionType::Parametric(
            ParametricInstructionType::Select,
        ));
        self.write_variable(i.input_vals[0]);
        self.write_variable(i.input_vals[1]);
        self.write_variable(i.select_val);
        self.write_variable(i.out1);
    }

    fn write_table_set(&mut self, i: TableSetInstruction) {
        self.write_instruction_type(InstructionType::Table(TableInstructionCategory::Set));
        self.write_immediate(i.table_idx);
        self.write_value_type(i.input_type);
        self.write_variable(i.in1);
        self.write_variable(i.idx);
    }

    fn write_table_get(&mut self, i: TableGetInstruction) {
        self.write_instruction_type(InstructionType::Table(TableInstructionCategory::Get));
        self.write_immediate(i.table_idx);
        self.write_variable(i.idx);
        self.write_variable(i.out1);
    }

    fn write_table_grow(&mut self, i: TableGrowInstruction) {
        self.write_instruction_type(InstructionType::Table(TableInstructionCategory::Grow));
        self.write_immediate(i.table_idx);
        self.write_variable(i.size);
        self.write_variable(i.value_to_fill);
        self.write_variable(i.out1);
    }

    fn write_table_size(&mut self, i: TableSizeInstruction) {
        self.write_instruction_type(InstructionType::Table(TableInstructionCategory::Size));
        self.write_immediate(i.table_idx);
        self.write_variable(i.out1);
    }

    fn write_table_fill(&mut self, i: TableFillInstruction) {
        self.write_instruction_type(InstructionType::Table(TableInstructionCategory::Fill));
        self.write_immediate(i.table_idx);
        self.write_variable(i.i);
        self.write_variable(i.n);
        self.write_variable(i.ref_value);
    }

    fn write_table_copy(&mut self, i: TableCopyInstruction) {
        self.write_instruction_type(InstructionType::Table(TableInstructionCategory::Copy));
        self.write_immediate(i.table_idx_x);
        self.write_immediate(i.table_idx_y);
        self.write_variable(i.n);
        self.write_variable(i.s);
        self.write_variable(i.d);
    }

    fn write_table_init(&mut self, i: TableInitInstruction) {
        self.write_instruction_type(InstructionType::Table(TableInstructionCategory::Init));
        self.write_immediate(i.table_idx);
        self.write_immediate(i.elem_idx);
        self.write_variable(i.n);
        self.write_variable(i.s);
        self.write_variable(i.d);
    }

    fn write_elem_drop(&mut self, i: ElemDropInstruction) {
        self.write_instruction_type(InstructionType::Table(TableInstructionCategory::Drop));
        self.write_immediate(i.elem_idx);
    }

    fn write_local_get(&mut self, i: LocalGetInstruction) {
        self.write_instruction_type(InstructionType::Variable(VariableInstructionType::LocalGet));
        self.write_immediate(i.local_idx);
        self.write_variable(i.out1);
    }

    fn write_global_get(&mut self, i: GlobalGetInstruction) {
        self.write_instruction_type(InstructionType::Variable(
            VariableInstructionType::GlobalGet,
        ));
        self.write_immediate(i.global_idx);
        self.write_variable(i.out1);
    }

    fn write_local_set(&mut self, i: LocalSetInstruction) {
        self.write_instruction_type(InstructionType::Variable(VariableInstructionType::LocalSet));
        self.write_immediate(i.local_idx);
        self.write_variable(i.in1);
    }

    fn write_global_set(&mut self, i: GlobalSetInstruction) {
        self.write_instruction_type(InstructionType::Variable(
            VariableInstructionType::GlobalSet,
        ));
        self.write_immediate(i.global_idx);
        self.write_variable(i.in1);
    }

    fn write_local_tee(&mut self, i: LocalTeeInstruction) {
        self.write_instruction_type(InstructionType::Variable(VariableInstructionType::LocalTee));
        self.write_immediate(i.local_idx);
        self.write_variable(i.in1);
        self.write_variable(i.out1);
    }

    fn finish(&mut self, terminator: ControlInstruction) {
        self.storage.terminator = terminator;
        self.finished = true;
    }

    fn peek_terminator(&self) -> &ControlInstruction {
        &self.storage.terminator
    }
}
