use super::*;
use crate::parser::parsable::Parse;
use wasm_types::*;

#[derive(Debug, Clone)]
pub(crate) struct LocalGetInstruction {
    local_idx: LocalIdx,
    out1: VariableID,
}

impl Instruction for LocalGetInstruction {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.write_instruction_type(InstructionType::Variable(VariableInstructionType::LocalGet));
        o.write_immediate(self.local_idx);
        o.write_variable(self.out1);
    }

    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let local_idx = i.read_immediate()?;
        let out1 = i.read_variable()?;
        Ok(LocalGetInstruction { local_idx, out1 })
    }
}

impl Display for LocalGetInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "%{} = local.get(i32 {})", self.out1, self.local_idx)
    }
}

pub(crate) fn local_get(
    ctxt: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let local_idx = LocalIdx::parse(i)?;
    let var = match ctxt.func.locals.get(local_idx as usize) {
        Some(v) => v,
        None => {
            ctxt.poison::<()>(ValidationError::Msg(format!(
                "local variable with index {} not found",
                local_idx
            )));
            return Ok(());
        }
    };

    let out = ctxt.create_var(var.type_);
    o.write(LocalGetInstruction {
        out1: out.id,
        local_idx,
    });
    ctxt.push_var(out);
    Ok(())
}

#[derive(Debug, Clone)]
pub(crate) struct GlobalGetInstruction {
    global_idx: GlobalIdx,
    out1: VariableID,
}

impl Instruction for GlobalGetInstruction {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.write_instruction_type(InstructionType::Variable(
            VariableInstructionType::GlobalGet,
        ));
        o.write_immediate(self.global_idx);
        o.write_variable(self.out1);
    }

    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let global_idx = i.read_immediate()?;
        let out1 = i.read_variable()?;
        Ok(GlobalGetInstruction { global_idx, out1 })
    }
}

impl Display for GlobalGetInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "%{} = global.get(i32 {})", self.out1, self.global_idx)
    }
}

pub(crate) fn global_get(
    ctxt: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let global_idx = GlobalIdx::parse(i)?;
    let global = match ctxt.module.globals.get(global_idx as usize) {
        Some(g) => g,
        None => {
            ctxt.poison::<()>(ValidationError::Msg(format!(
                "global variable with index {} not found",
                global_idx
            )));
            return Ok(());
        }
    };

    let global_type = match &global.r#type {
        GlobalType::Const(t) => t,
        GlobalType::Mut(t) => t,
    };
    let out = ctxt.create_var(*global_type);
    o.write(GlobalGetInstruction {
        out1: out.id,
        global_idx,
    });
    ctxt.push_var(out);
    Ok(())
}

#[derive(Debug, Clone)]
pub(crate) struct LocalSetInstruction {
    local_idx: LocalIdx,
    in1: VariableID,
}

impl Instruction for LocalSetInstruction {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.write_instruction_type(InstructionType::Variable(VariableInstructionType::LocalSet));
        o.write_immediate(self.local_idx);
        o.write_variable(self.in1);
    }

    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let local_idx = i.read_immediate()?;
        let in1 = i.read_variable()?;
        Ok(LocalSetInstruction { local_idx, in1 })
    }
}

impl Display for LocalSetInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "local.set(i32 {}) %{}", self.local_idx, self.in1)
    }
}

pub(crate) fn local_set(
    ctxt: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let local_idx = LocalIdx::parse(i)?;
    let var = match ctxt.func.locals.get(local_idx as usize) {
        Some(v) => v,
        None => {
            ctxt.poison::<()>(ValidationError::Msg(format!(
                "local variable with index {} not found",
                local_idx
            )));
            return Ok(());
        }
    };

    let value = ctxt.pop_var_with_type(&var.type_);
    if value.type_ != var.type_ {
        ctxt.poison(ValidationError::Msg(format!(
            "stack value type does not match local {} type: {:?} vs {:?}",
            local_idx, value.type_, var.type_
        )))
    }

    o.write(LocalSetInstruction {
        local_idx,
        in1: value.id,
    });
    Ok(())
}

#[derive(Debug, Clone)]
pub(crate) struct GlobalSetInstruction {
    global_idx: GlobalIdx,
    in1: VariableID,
}

impl Instruction for GlobalSetInstruction {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.write_instruction_type(InstructionType::Variable(
            VariableInstructionType::GlobalSet,
        ));
        o.write_immediate(self.global_idx);
        o.write_variable(self.in1);
    }

    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let global_idx = i.read_immediate()?;
        let in1 = i.read_variable()?;
        Ok(GlobalSetInstruction { global_idx, in1 })
    }
}

impl Display for GlobalSetInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "global.set(i32 {}) %{}", self.global_idx, self.in1)
    }
}

pub(crate) fn global_set(
    ctxt: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let global_idx = GlobalIdx::parse(i)?;
    let global = match ctxt.module.globals.get(global_idx as usize) {
        Some(g) => g,
        None => {
            ctxt.poison::<()>(ValidationError::Msg(format!(
                "global variable with index {} not found",
                global_idx
            )));
            return Ok(());
        }
    };

    let global_type = match &global.r#type {
        GlobalType::Const(_) => {
            ctxt.poison::<()>(ValidationError::Msg(format!(
                "cannot set const global {}",
                global_idx
            )));
            return Ok(());
        }
        GlobalType::Mut(t) => t,
    };

    let value = ctxt.pop_var_with_type(global_type);
    if value.type_ != *global_type {
        ctxt.poison(ValidationError::Msg(format!(
            "stack value type does not match global {} type: {:?} vs {:?}",
            global_idx, value.type_, global_type
        )))
    }

    o.write(GlobalSetInstruction {
        global_idx,
        in1: value.id,
    });
    Ok(())
}

#[derive(Debug, Clone)]
pub(crate) struct TeeLocalInstruction {
    local_idx: LocalIdx,
    in1: VariableID,
    out1: VariableID,
}

impl Instruction for TeeLocalInstruction {
    fn serialize(self, o: &mut InstructionEncoder) {
        o.write_instruction_type(InstructionType::Variable(VariableInstructionType::LocalTee));
        o.write_immediate(self.local_idx);
        o.write_variable(self.in1);
        o.write_variable(self.out1);
    }

    fn deserialize(i: &mut InstructionDecoder, _: InstructionType) -> Result<Self, DecodingError> {
        let local_idx = i.read_immediate()?;
        let in1 = i.read_variable()?;
        let out1 = i.read_variable()?;
        Ok(TeeLocalInstruction {
            local_idx,
            in1,
            out1,
        })
    }
}

impl Display for TeeLocalInstruction {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "%{} = local.tee(i32 {}) %{}",
            self.out1, self.local_idx, self.in1
        )
    }
}

pub(crate) fn local_tee(
    ctxt: &mut Context,
    i: &mut WasmStreamReader,
    o: &mut InstructionEncoder,
) -> ParseResult {
    let local_idx = LocalIdx::parse(i)?;
    let local_var = match ctxt.func.locals.get(local_idx as usize) {
        Some(v) => v,
        None => {
            ctxt.poison::<()>(ValidationError::Msg(format!(
                "local variable with index {} not found",
                local_idx
            )));
            return Ok(());
        }
    };

    let in_stack_var = ctxt.pop_var_with_type(&local_var.type_);
    if in_stack_var.type_ != local_var.type_ {
        ctxt.poison(ValidationError::Msg(format!(
            "stack value type does not match local {} type: {:?} vs {:?}",
            local_idx, in_stack_var.type_, local_var.type_
        )))
    }

    let out_stack_var = ctxt.create_var(local_var.type_);

    o.write(TeeLocalInstruction {
        local_idx,
        in1: in_stack_var.id,
        out1: out_stack_var.id,
    });

    ctxt.push_var(out_stack_var);
    Ok(())
}
