use crate::RuntimeError;
use ir::{
    structs::value::{Number, Value},
    utils::numeric_transmutes::{Bit32, Bit64},
};
use wasm_types::{FuncType, NumType, ValType};

macro_rules! macro_invoke_for_each_function_signature {
    ($macro:ident) => {
        $macro!(0);
        $macro!(1 Param1);
        $macro!(2 Param1 Param2);
        $macro!(3 Param1 Param2 Param3);
        $macro!(4 Param1 Param2 Param3 Param4);
        $macro!(5 Param1 Param2 Param3 Param4 Param5);
        $macro!(6 Param1 Param2 Param3 Param4 Param5 Param6);
        $macro!(7 Param1 Param2 Param3 Param4 Param5 Param6 Param7);
        $macro!(8 Param1 Param2 Param3 Param4 Param5 Param6 Param7 Param8);
        $macro!(9 Param1 Param2 Param3 Param4 Param5 Param6 Param7 Param8 Param9);
        $macro!(10 Param1 Param2 Param3 Param4 Param5 Param6 Param7 Param8 Param9 Param10);
        $macro!(11 Param1 Param2 Param3 Param4 Param5 Param6 Param7 Param8 Param9 Param10 Param11);
        $macro!(12 Param1 Param2 Param3 Param4 Param5 Param6 Param7 Param8 Param9 Param10 Param11 Param12);
        $macro!(13 Param1 Param2 Param3 Param4 Param5 Param6 Param7 Param8 Param9 Param10 Param11 Param12 Param13);
        $macro!(14 Param1 Param2 Param3 Param4 Param5 Param6 Param7 Param8 Param9 Param10 Param11 Param12 Param13 Param14);
        $macro!(15 Param1 Param2 Param3 Param4 Param5 Param6 Param7 Param8 Param9 Param10 Param11 Param12 Param13 Param14 Param15);
        $macro!(16 Param1 Param2 Param3 Param4 Param5 Param6 Param7 Param8 Param9 Param10 Param11 Param12 Param13 Param14 Param15 Param16);
    };
}
pub(crate) use macro_invoke_for_each_function_signature;

#[derive(Clone)]
pub(crate) enum Either<T1, T2> {
    Left(T1),
    Right(T2),
}

pub(crate) unsafe fn super_unsafe_copy_to_ref_mut<T>(r: &T) -> &'static mut T {
    &mut core::slice::from_raw_parts_mut(r as *const _ as *mut _, 1)[0]
}

fn parse_input_params_for_function(function_type: &FuncType) -> Result<Vec<Value>, RuntimeError> {
    let args = std::env::args().skip(2);
    let num_args = args.len();
    if num_args != function_type.num_params() {
        return Err(RuntimeError::ArgumentNumberMismatch(
            function_type.num_params(),
            num_args,
        ));
    }
    let mut values = Vec::new();
    for (param_type, arg) in function_type.params_iter().zip(args) {
        let value = match param_type {
            ValType::Number(NumType::F32) => {
                Value::Number(Number::F32(arg.parse().map_err(|_| {
                    RuntimeError::InvalidArgumentType(param_type.to_owned(), arg)
                })?))
            }
            ValType::Number(NumType::F64) => {
                Value::Number(Number::F64(arg.parse().map_err(|_| {
                    RuntimeError::InvalidArgumentType(param_type.to_owned(), arg)
                })?))
            }
            ValType::Number(NumType::I32) => {
                let value: u32 = arg.parse::<u32>().or_else(|_| {
                    arg.parse::<i32>()
                        .map_err(|_| RuntimeError::InvalidArgumentType(param_type.to_owned(), arg))
                        .map(|i| i.trans_u32())
                })?;
                Value::Number(Number::I32(value))
            }
            ValType::Number(NumType::I64) => {
                let value: u64 = arg.parse::<u64>().or_else(|_| {
                    arg.parse::<i64>()
                        .map_err(|_| RuntimeError::InvalidArgumentType(param_type.to_owned(), arg))
                        .map(|i| i.trans_u64())
                })?;
                Value::Number(Number::I64(value))
            }
            _ => {
                return Err(RuntimeError::InvalidArgumentType(
                    param_type.to_owned(),
                    arg,
                ))
            }
        };
        values.push(value);
    }
    Ok(values)
}
