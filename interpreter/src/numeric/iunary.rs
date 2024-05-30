use wasm_types::{IUnaryOp, NumType};
use {
    crate::{
        Executable, InterpreterContext,
        InterpreterError::{self, TypeMismatch},
        StackFrame,
    },
    ir::instructions::IUnaryInstruction,
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
    let types = todo!();
    let leading_zeros = match types {
        // be careful here, this works for u64 -> u32, but might not work for other types.
        NumType::I32 => u32::leading_zeros(num as u32),
        NumType::I64 => num.leading_zeros(),
        _ => return Err(TypeMismatch),
    };

    stack_frame.vars.set(instruction.out1, leading_zeros as u64);

    Ok(())
}

fn ctz(
    stack_frame: &mut StackFrame,
    instruction: &mut IUnaryInstruction,
) -> Result<(), InterpreterError> {
    let num = stack_frame.vars.get(instruction.in1);
    let types = todo!();
    let trailing_zeros = match types {
        NumType::I32 => u32::trailing_zeros(num as u32),
        NumType::I64 => num.trailing_zeros(),
        _ => return Err(TypeMismatch),
    };

    stack_frame
        .vars
        .set(instruction.out1, trailing_zeros as u64);

    Ok(())
}

fn popcnt(
    stack_frame: &mut StackFrame,
    instruction: &mut IUnaryInstruction,
) -> Result<(), InterpreterError> {
    let num = stack_frame.vars.get(instruction.in1);
    let types = todo!();

    let pop_count = match types {
        NumType::I32 => u32::count_ones(num as u32),
        NumType::I64 => num.count_ones(),
        _ => return Err(TypeMismatch),
    };

    stack_frame.vars.set(instruction.out1, pop_count as u64);

    Ok(())
}
