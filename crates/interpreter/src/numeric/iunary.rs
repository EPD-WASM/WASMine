use wasm_types::{IUnaryOp, NumType};
use {
    crate::{
        Executable, InterpreterContext,
        InterpreterError::{self, TypeMismatch},
        StackFrame,
    },
    module::instructions::IUnaryInstruction,
};

impl Executable for IUnaryInstruction {
    fn execute(&mut self, ctx: &mut InterpreterContext) -> Result<(), InterpreterError> {
        let stack_frame = ctx.stack.last_mut().unwrap();
        match &self.op {
            IUnaryOp::Clz => clz(stack_frame, self)?,
            IUnaryOp::Ctz => ctz(stack_frame, self)?,
            IUnaryOp::Popcnt => popcnt(stack_frame, self)?,
        }

        Ok(())
    }
}

fn clz(
    stack_frame: &mut StackFrame,
    instruction: &mut IUnaryInstruction,
) -> Result<(), InterpreterError> {
    let num = stack_frame.vars.get(instruction.in1);

    let leading_zeros = match instruction.types {
        // be careful here, this works for u64 -> u32, but might not work for other types.
        NumType::I32 => u32::leading_zeros(num.as_u32()),
        NumType::I64 => num.as_u64().leading_zeros(),
        _ => return Err(TypeMismatch),
    } as u64;

    stack_frame.vars.set(instruction.out1, leading_zeros.into());

    Ok(())
}

fn ctz(
    stack_frame: &mut StackFrame,
    instruction: &mut IUnaryInstruction,
) -> Result<(), InterpreterError> {
    let num = stack_frame.vars.get(instruction.in1);

    let trailing_zeros = match instruction.types {
        NumType::I32 => u32::trailing_zeros(num.as_u32()),
        NumType::I64 => num.as_u64().trailing_zeros(),
        _ => return Err(TypeMismatch),
    } as u64;

    stack_frame
        .vars
        .set(instruction.out1, trailing_zeros.into());

    Ok(())
}

fn popcnt(
    stack_frame: &mut StackFrame,
    instruction: &mut IUnaryInstruction,
) -> Result<(), InterpreterError> {
    let num = stack_frame.vars.get(instruction.in1);

    let pop_count = match instruction.types {
        NumType::I32 => u32::count_ones(num.as_u32()),
        NumType::I64 => num.as_u32().count_ones(),
        _ => return Err(TypeMismatch),
    } as u64;

    stack_frame.vars.set(instruction.out1, pop_count.into());

    Ok(())
}
