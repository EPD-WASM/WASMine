use ir::{
    structs::value::Value,
    utils::numeric_transmutes::{Bit32, Bit64},
};
use runtime_lib::RuntimeError;
use wasm_types::{FuncType, NumType, ValType};

pub(crate) fn parse_input_params_for_function(
    args: Vec<String>,
    function_type: FuncType,
) -> Result<Vec<Value>, RuntimeError> {
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
            ValType::Number(NumType::F32) => Value::f32(
                arg.parse()
                    .map_err(|_| RuntimeError::InvalidArgumentType(param_type.to_owned(), arg))?,
            ),
            ValType::Number(NumType::F64) => Value::f64(
                arg.parse()
                    .map_err(|_| RuntimeError::InvalidArgumentType(param_type.to_owned(), arg))?,
            ),
            ValType::Number(NumType::I32) => {
                let value: u32 = arg.parse::<u32>().or_else(|_| {
                    arg.parse::<i32>()
                        .map_err(|_| RuntimeError::InvalidArgumentType(param_type.to_owned(), arg))
                        .map(|i| i.trans_u32())
                })?;
                Value::i32(value)
            }
            ValType::Number(NumType::I64) => {
                let value: u64 = arg.parse::<u64>().or_else(|_| {
                    arg.parse::<i64>()
                        .map_err(|_| RuntimeError::InvalidArgumentType(param_type.to_owned(), arg))
                        .map(|i| i.trans_u64())
                })?;
                Value::i64(value)
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
