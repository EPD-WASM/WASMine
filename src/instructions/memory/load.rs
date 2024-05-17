use super::*;

#[derive(Debug, Clone)]
pub(crate) struct LoadInstruction {
    memarg: MemArg,
    out1: VariableID,
    out1_type: NumType,
    addr: VariableID,
    operation: LoadOp,
}

impl Instruction for LoadInstruction {
    fn serialize(self, o: &mut O) {
        o.write_instruction_type(InstructionType::Memory(MemoryInstructionCategory::Load(
            self.operation.clone(),
        )));
        o.write_immediate(self.memarg.align);
        o.write_immediate(self.memarg.offset);
        o.write_variable(self.addr);
        o.write_variable(self.out1);
        o.write_value_type(ValType::Number(self.out1_type))
    }

    fn deserialize(i: &mut InstructionDecoder, t: InstructionType) -> Result<Self, DecodingError> {
        let align = i.read_immediate()?;
        let offset = i.read_immediate()?;
        let addr = i.read_variable()?;
        let out1 = i.read_variable()?;
        let out1_type = extract_numtype!(i.read_value_type()?);
        let operation = match t {
            InstructionType::Memory(MemoryInstructionCategory::Load(op)) => op,
            _ => return Err(DecodingError::TypeMismatch),
        };
        Ok(LoadInstruction {
            memarg: MemArg { align, offset },
            out1,
            out1_type,
            addr,
            operation,
        })
    }
}

fn parse_load(
    ctxt: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
    out_type: NumType,
    operation: LoadOp,
) -> ParseResult {
    let memarg = MemArg::parse(i)?;
    let in_ = ctxt.pop_var_with_type(&ValType::Number(NumType::I32));
    let out = ctxt.create_var(ValType::Number(out_type));
    o.write(LoadInstruction {
        memarg,
        out1: out.id,
        out1_type: out_type,
        addr: in_.id,
        operation,
    });
    ctxt.push_var(out);
    Ok(())
}

#[rustfmt::skip]
mod load_specializations {
    use super::*;
    pub(crate) fn i32_load(c: &mut C, i: &mut I, o: &mut O) -> PR {parse_load(c, i, o, NumType::I32, LoadOp::INNLoad)}
    pub(crate) fn i64_load(c: &mut C, i: &mut I, o: &mut O) -> PR {parse_load(c, i, o, NumType::I64, LoadOp::INNLoad)}
    pub(crate) fn f32_load(c: &mut C, i: &mut I, o: &mut O) -> PR {parse_load(c, i, o, NumType::F32, LoadOp::FNNLoad)}
    pub(crate) fn f64_load(c: &mut C, i: &mut I, o: &mut O) -> PR {parse_load(c, i, o, NumType::F64, LoadOp::FNNLoad)}

    pub(crate) fn i32_load8_s(c: &mut C, i: &mut I, o: &mut O) -> PR {parse_load(c, i, o, NumType::I32, LoadOp::INNLoad8S)}
    pub(crate) fn i32_load8_u(c: &mut C, i: &mut I, o: &mut O) -> PR {parse_load(c, i, o, NumType::I32, LoadOp::INNLoad8U)}
    pub(crate) fn i32_load16_s(c: &mut C, i: &mut I, o: &mut O) -> PR {parse_load(c, i, o, NumType::I32, LoadOp::INNLoad16S)}
    pub(crate) fn i32_load16_u(c: &mut C, i: &mut I, o: &mut O) -> PR {parse_load(c, i, o, NumType::I32, LoadOp::INNLoad16U)}

    pub(crate) fn i64_load8_s(c: &mut C, i: &mut I, o: &mut O) -> PR {parse_load(c, i, o, NumType::I64, LoadOp::INNLoad8S)}
    pub(crate) fn i64_load8_u(c: &mut C, i: &mut I, o: &mut O) -> PR {parse_load(c, i, o, NumType::I64, LoadOp::INNLoad8U)}
    pub(crate) fn i64_load16_s(c: &mut C, i: &mut I, o: &mut O) -> PR {parse_load(c, i, o, NumType::I64, LoadOp::INNLoad16S)}
    pub(crate) fn i64_load16_u(c: &mut C, i: &mut I, o: &mut O) -> PR {parse_load(c, i, o, NumType::I64, LoadOp::INNLoad16U)}
    pub(crate) fn i64_load32_s(c: &mut C, i: &mut I, o: &mut O) -> PR {parse_load(c, i, o, NumType::I64, LoadOp::INNLoad32S)}
    pub(crate) fn i64_load32_u(c: &mut C, i: &mut I, o: &mut O) -> PR {parse_load(c, i, o, NumType::I64, LoadOp::INNLoad32U)}
}
pub(crate) use load_specializations::*;

use crate::{
    structs::memory::MemArg,
    wasm_types::{InstructionType, LoadOp, MemoryInstructionCategory, NumType},
};

use super::VariableID;
