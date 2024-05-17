use super::*;
use crate::{structs::value::Number, wasm_types::*};

#[derive(Debug, Clone)]
pub(crate) struct Constant {
    storage: Number,
    out1: VariableID,
    out1_type: NumType,
}

impl Instruction for Constant {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.write_instruction_type(InstructionType::Numeric(
            NumericInstructionCategory::Constant,
        ));
        match self.storage {
            Number::U32(imm) => {
                o.write_immediate(imm);
            }
            Number::U64(imm) => {
                o.write_immediate(imm);
            }
            Number::S32(imm) => {
                o.write_immediate(imm);
            }
            Number::S64(imm) => {
                o.write_immediate(imm);
            }
            Number::I32(imm) => {
                o.write_immediate(imm);
            }
            Number::I64(imm) => {
                o.write_immediate(imm);
            }
            Number::F32(imm) => {
                o.write_immediate_float32(imm);
            }
            Number::F64(imm) => {
                o.write_immediate_float64(imm);
            }
        }
        o.write_variable(self.out1);
        o.write_value_type(ValType::Number(self.out1_type));
    }

    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let num_type = extract_numtype!(i.read_value_type()?);
        let imm = match num_type {
            NumType::I32 => Number::S32(i.read_immediate()?),
            NumType::I64 => Number::S64(i.read_immediate()?),
            NumType::F32 => Number::F32(i.read_immediate_float32()?),
            NumType::F64 => Number::F64(i.read_immediate_float64()?),
        };
        Ok(Constant {
            storage: imm,
            out1: i.read_variable()?,
            out1_type: num_type,
        })
    }
}

pub(crate) fn i32_const_i32(ctxt: &mut C, i: &mut I, o: &mut O) -> PR {
    let imm = i.read_leb128::<i32>()?;
    let imm_var = ctxt.create_var(ValType::Number(NumType::I32));
    let const_instr = Constant {
        storage: Number::I32(unsafe { std::mem::transmute::<i32, u32>(imm) }),
        out1: imm_var.id,
        out1_type: NumType::I32,
    };
    o.write(const_instr);
    ctxt.push_var(imm_var);
    Ok(())
}

pub(crate) fn i64_const_i64(ctxt: &mut C, i: &mut I, o: &mut O) -> PR {
    let imm = i.read_leb128::<i64>()?;
    let imm_var = ctxt.create_var(ValType::Number(NumType::I64));
    let const_instr = Constant {
        storage: Number::I64(unsafe { std::mem::transmute::<i64, u64>(imm) }),
        out1: imm_var.id,
        out1_type: NumType::I64,
    };
    o.write(const_instr);
    ctxt.push_var(imm_var);
    Ok(())
}

pub(crate) fn f32_const_f32(ctxt: &mut C, i: &mut I, o: &mut O) -> PR {
    let imm = i.read_f32()?;
    let imm_var = ctxt.create_var(ValType::Number(NumType::F32));
    let const_instr = Constant {
        storage: Number::F32(imm),
        out1: imm_var.id,
        out1_type: NumType::F32,
    };
    o.write(const_instr);
    ctxt.push_var(imm_var);
    Ok(())
}

pub(crate) fn f64_const_f64(ctxt: &mut C, i: &mut I, o: &mut O) -> PR {
    let imm = i.read_f64()?;
    let imm_var = ctxt.create_var(ValType::Number(NumType::F64));
    let const_instr = Constant {
        storage: Number::F64(imm),
        out1: imm_var.id,
        out1_type: NumType::F64,
    };
    o.write(const_instr);
    ctxt.push_var(imm_var);
    Ok(())
}
