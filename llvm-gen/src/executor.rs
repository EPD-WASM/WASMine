use crate::{
    abstraction::{context::Context, execution_engine::ExecutionEngine, module::Module},
    error::ExecutionError,
};
use ir::{
    structs::value::{Number, Reference, Value},
    utils::numeric_transmutes::Bit64,
};
use runtime_interface::{ExecutionContext, RawFunctionPtr};
use std::rc::Rc;
use wasm_types::{GlobalIdx, NumType, RefType, ResType, ValType};

pub struct Executor {
    execution_engine: ExecutionEngine,
    #[allow(dead_code)] // hold on to the context to prevent it from being dropped
    context: Rc<Context>,
}

type FunVoid = unsafe extern "C" fn(*mut ExecutionContext, *const u64) -> ();
type FunU32 = unsafe extern "C" fn(*mut ExecutionContext, *const u64) -> u32;
type FunU64 = unsafe extern "C" fn(*mut ExecutionContext, *const u64) -> u64;
type FunF32 = unsafe extern "C" fn(*mut ExecutionContext, *const u64) -> f32;
type FunF64 = unsafe extern "C" fn(*mut ExecutionContext, *const u64) -> f64;
type FunMultiRet = unsafe extern "C" fn(*mut ExecutionContext, *const u64, *const u64) -> ();

impl Executor {
    pub fn new(context: Rc<Context>) -> Result<Self, ExecutionError> {
        Ok(Self {
            execution_engine: ExecutionEngine::init()?,
            context,
        })
    }

    pub fn get_raw_by_name(&self, function_name: &str) -> Result<RawFunctionPtr, ExecutionError> {
        self.execution_engine
            .find_func_address_by_name(function_name)
    }

    pub fn register_symbol(&mut self, name: &str, address: RawFunctionPtr) {
        self.execution_engine.register_symbol(name, address);
    }

    pub fn add_module(&mut self, module: Rc<Module>) -> Result<(), ExecutionError> {
        self.execution_engine.optimize_module(&module)?;

        #[cfg(debug_assertions)]
        module.print_to_file();

        self.execution_engine.add_llvm_module(module)
    }

    pub fn get_global_value(&self, global_idx: GlobalIdx) -> Result<u64, ExecutionError> {
        self.execution_engine
            .get_global_value(&format!("global_{}", global_idx))
    }

    /// # Safety
    /// This function is unsafe because it dereferences the `exec_ctxt` pointer.
    pub unsafe fn run(
        &mut self,
        func_name: &str,
        func_ret_type: ResType,
        parameters: Vec<Value>,
        exec_ctxt: *mut ExecutionContext,
    ) -> Result<Vec<Value>, ExecutionError> {
        let parameters = parameters
            .into_iter()
            .map(|v| match v {
                Value::Number(n) => n.trans_to_u64(),
                Value::Reference(Reference::Null) => u64::MAX,
                Value::Reference(Reference::Extern(r)) => r as u64,
                Value::Reference(Reference::Function(r)) => r as u64,
                Value::Vector(_) => todo!(),
            })
            .collect::<Vec<u64>>();
        match func_ret_type.as_slice() {
            [] => {
                let compiled_func_addr: FunVoid =
                    self.execution_engine.find_func_by_name(func_name)?;
                let _: () = unsafe { (compiled_func_addr)(exec_ctxt, parameters.as_ptr()) };
                Ok(vec![])
            }
            [ValType::Number(NumType::I32)] => {
                let compiled_func_addr: FunU32 =
                    self.execution_engine.find_func_by_name(func_name)?;
                let result: u32 = unsafe { (compiled_func_addr)(exec_ctxt, parameters.as_ptr()) };
                Ok(vec![Value::Number(Number::I32(result))])
            }
            [ValType::Number(NumType::I64)] => {
                let compiled_func_addr: FunU64 =
                    self.execution_engine.find_func_by_name(func_name)?;
                let result: u64 = unsafe { (compiled_func_addr)(exec_ctxt, parameters.as_ptr()) };
                Ok(vec![Value::Number(Number::I64(result))])
            }
            [ValType::Number(NumType::F32)] => {
                let compiled_func_addr: FunF32 =
                    self.execution_engine.find_func_by_name(func_name)?;
                let result: f32 = unsafe { (compiled_func_addr)(exec_ctxt, parameters.as_ptr()) };
                Ok(vec![Value::Number(Number::F32(result))])
            }
            [ValType::Number(NumType::F64)] => {
                let compiled_func_addr: FunF64 =
                    self.execution_engine.find_func_by_name(func_name)?;
                let result: f64 = unsafe { (compiled_func_addr)(exec_ctxt, parameters.as_ptr()) };
                Ok(vec![Value::Number(Number::F64(result))])
            }
            [ValType::Reference(RefType::ExternReference)] => {
                let compiled_func_addr: FunU64 =
                    self.execution_engine.find_func_by_name(func_name)?;
                let result: u64 = unsafe { (compiled_func_addr)(exec_ctxt, parameters.as_ptr()) };
                if result == Value::Reference(Reference::Null).to_generic() {
                    Ok(vec![Value::Reference(Reference::Null)])
                } else {
                    Ok(vec![Value::Reference(Reference::Extern(result as _))])
                }
            }
            [ValType::Reference(RefType::FunctionReference)] => {
                let compiled_func_addr: FunU64 =
                    self.execution_engine.find_func_by_name(func_name)?;
                let result: u64 = unsafe { (compiled_func_addr)(exec_ctxt, parameters.as_ptr()) };
                if result == Value::Reference(Reference::Null).to_generic() {
                    Ok(vec![Value::Reference(Reference::Null)])
                } else {
                    Ok(vec![Value::Reference(Reference::Function(
                        result.trans_u32(),
                    ))])
                }
            }
            [ValType::VecType] => todo!(),
            _ => {
                let return_vals = func_ret_type.iter().map(|_| 0).collect::<Vec<u64>>();
                let compiled_func_addr: FunMultiRet =
                    self.execution_engine.find_func_by_name(func_name)?;
                let _: () = unsafe {
                    (compiled_func_addr)(exec_ctxt, parameters.as_ptr(), return_vals.as_ptr())
                };
                Ok(return_vals
                    .into_iter()
                    .zip(func_ret_type)
                    .map(|(val, val_type)| Value::from_generic(val_type, val))
                    .collect())
            }
        }
    }
}
