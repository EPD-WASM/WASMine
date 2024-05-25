use super::*;
use crate::parser::parsable::Parse;

#[derive(Debug, Clone)]
pub struct ReferenceIsNullInstruction {
    in1: VariableID,
    in1_type: ValType,
    out1: VariableID,
}

impl Instruction for ReferenceIsNullInstruction {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.write_instruction_type(InstructionType::Reference(
            ReferenceInstructionType::RefIsNull,
        ));
        o.write_variable(self.in1);
        o.write_value_type(self.in1_type);
        o.write_variable(self.out1);
    }

    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let in1 = i.read_variable()?;
        let in1_type = i.read_value_type()?;
        let out1 = i.read_variable()?;
        Ok(ReferenceIsNullInstruction {
            in1,
            in1_type,
            out1,
        })
    }
}

pub(crate) fn ref_is_null(
    ctxt: &mut Context,
    _: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let val = ctxt.pop_var();
    match val.type_ {
        ValType::Reference(_) => {}
        _ => ctxt.poison(ValidationError::Msg(
            "ref.is_null expects a reference type on stack".into(),
        )),
    }
    let out = ctxt.create_var(ValType::Number(NumType::I32));
    o.write(ReferenceIsNullInstruction {
        in1: val.id,
        in1_type: val.type_,
        out1: out.id,
    });
    ctxt.push_var(out);
    Ok(())
}

#[derive(Debug, Clone)]
pub struct ReferenceNullInstruction {
    out1: VariableID,
    out1_type: ValType,
}

impl Instruction for ReferenceNullInstruction {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.write_instruction_type(InstructionType::Reference(
            ReferenceInstructionType::RefNull,
        ));
        o.write_variable(self.out1);
        o.write_value_type(self.out1_type);
    }

    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let out1 = i.read_variable()?;
        let out1_type = i.read_value_type()?;
        Ok(ReferenceNullInstruction { out1, out1_type })
    }
}

pub(crate) fn ref_null(
    ctxt: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let ref_type = RefType::parse(i)?;
    let out = ctxt.create_var(ValType::Reference(ref_type));
    o.write(ReferenceNullInstruction {
        out1: out.id,
        out1_type: ValType::Reference(ref_type),
    });
    ctxt.push_var(out);
    Ok(())
}

#[derive(Debug, Clone)]
pub struct ReferenceFunctionInstruction {
    out1: VariableID,
    func_idx: FuncIdx,
}

impl Instruction for ReferenceFunctionInstruction {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.write_instruction_type(InstructionType::Reference(
            ReferenceInstructionType::RefFunc,
        ));
        o.write_variable(self.out1);
        o.write_immediate(self.func_idx);
    }

    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let out1 = i.read_variable()?;
        let func_idx = i.read_immediate()?;
        Ok(ReferenceFunctionInstruction { out1, func_idx })
    }
}

pub(crate) fn ref_func(
    ctxt: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let func_idx = FuncIdx::parse(i)?;
    let out = ctxt.create_var(ValType::Reference(RefType::FunctionReference));
    o.write(ReferenceFunctionInstruction {
        out1: out.id,
        func_idx,
    });
    ctxt.push_var(out);
    Ok(())
}
