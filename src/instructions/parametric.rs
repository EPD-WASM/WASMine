use super::*;
use crate::parser::parsable::Parse;
use wasm_types::*;

#[derive(Debug, Clone)]
pub(crate) struct DropInstruction {}

impl Instruction for DropInstruction {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.write_instruction_type(InstructionType::Parametric(ParametricInstructionType::Drop));
    }

    fn deserialize(_: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        Ok(DropInstruction {})
    }
}

impl Display for DropInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "nop (orig: drop)")
    }
}

pub(crate) fn drop(
    ctxt: &mut Context,
    _: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    ctxt.pop_var();
    o.write(DropInstruction {});
    Ok(())
}

#[derive(Debug, Clone)]
pub(crate) struct SelectInstruction {
    input_vals: [VariableID; 2],
    select_val: VariableID,
    out1: VariableID,
}

impl Instruction for SelectInstruction {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.write_instruction_type(InstructionType::Parametric(
            ParametricInstructionType::Select,
        ));
        o.write_variable(self.input_vals[0]);
        o.write_variable(self.input_vals[1]);
        o.write_variable(self.select_val);
        o.write_variable(self.out1);
    }

    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let input_vals = [i.read_variable()?, i.read_variable()?];
        let select_val = i.read_variable()?;
        let out1 = i.read_variable()?;
        Ok(SelectInstruction {
            input_vals,
            select_val,
            out1,
        })
    }
}

impl Display for SelectInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "%{} = select %{} %{} %{}",
            self.out1, self.input_vals[0], self.input_vals[1], self.select_val
        )
    }
}

pub(crate) fn select_numeric(
    ctxt: &mut Context,
    _: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let select_val = ctxt.pop_var_with_type(&ValType::Number(NumType::I32));

    let val1 = ctxt.pop_var();
    let val2 = ctxt.pop_var();
    if val1.type_ != val2.type_ {
        ctxt.poison(ValidationError::Msg(format!(
            "type mismatch for select: {:?} != {:?}",
            val1.type_, val2.type_
        )))
    }
    match &val1.type_ {
        ValType::Number(_) | ValType::VecType => {}
        _ => ctxt.poison(ValidationError::Msg(format!(
            "invalid type for numeric select: {:?}",
            val1.type_
        ))),
    }

    let out = ctxt.create_var(val1.type_);
    o.write(SelectInstruction {
        input_vals: [val1.id, val2.id],
        select_val: select_val.id,
        out1: out.id,
    });
    ctxt.push_var(out);
    Ok(())
}

pub(crate) fn select_generic(
    ctxt: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let num_val_types = i.read_leb128::<u32>()?;
    if num_val_types != 1 {
        ctxt.poison(ValidationError::Msg(format!(
            "invalid number of val types for select: {} != 1",
            num_val_types
        )))
    }
    let val_type = ValType::parse(i)?;
    let select_val = ctxt.pop_var_with_type(&ValType::Number(NumType::I32));

    let val1 = ctxt.pop_var_with_type(&val_type);
    let val2 = ctxt.pop_var_with_type(&val_type);
    let out = ctxt.create_var(val_type);
    o.write(SelectInstruction {
        input_vals: [val1.id, val2.id],
        select_val: select_val.id,
        out1: out.id,
    });
    ctxt.push_var(out);
    Ok(())
}
