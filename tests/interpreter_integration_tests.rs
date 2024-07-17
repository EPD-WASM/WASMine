use core::panic;
use gen_util::generate_spec_test_cases;
use ir::{
    function::{Function, FunctionSource},
    structs::{
        module::Module,
        value::{Number, Reference, Value},
    },
    utils::numeric_transmutes::{Bit32, Bit64},
};
use loader::Loader;
use parser::{error::ParserError, parser::Parser};
use runtime_lib::{BoundLinker, Cluster, Engine, InstanceHandle, Linker, RuntimeError};
use std::{collections::HashMap, path::PathBuf, rc::Rc};
use test_log::test;
use wasm_types::{NumType, ValType};
use wast::{
    core::{NanPattern, WastArgCore, WastRetCore},
    QuoteWat, Wast, WastExecute, Wat,
};

type DeclaredModulesMap<'a> = HashMap<
    Option<String>,
    (
        InstanceHandle<'a>,
        /* start_fun already called? */ bool,
    ),
>;

generate_spec_test_cases!(test_interpreter, interpreter);

pub fn translate_module<'a>(
    module: Rc<ir::structs::module::Module>,
    linker: &mut BoundLinker<'a>,
) -> Result<(InstanceHandle<'a>, bool), RuntimeError> {
    let mut engine = Engine::interpreter()?;
    engine.init(module.clone())?;
    let instance_handle = linker.instantiate_and_link(module.clone(), engine)?;
    Ok((instance_handle, false))
}

fn execute_via_interpreter_backend(
    file_path: &str,
    line: usize,
    col: usize,
    invoke: &wast::WastInvoke,
    declared_modules: &mut DeclaredModulesMap,
) -> Result<Vec<Value>, RuntimeError> {
    let prefix = format!("{:?}:{}:{}\n", file_path, line, col);
    let (instance, already_started) =
        match declared_modules.get_mut(&invoke.module.map(|m| m.name().to_owned())) {
            Some(m) => m,
            None => {
                panic!("{}Module not found: {:?}", prefix, invoke.module);
            }
        };
    if !*already_started {
        if let Some(entry_point) = instance.wasm_module().entry_point {
            // if the entry point is an import, we don't need to run it
            if !matches!(
                instance.wasm_module().ir.functions[entry_point as usize].src,
                FunctionSource::Import(_)
            ) {
                let fn_name = Function::query_function_name(entry_point, instance.wasm_module())
                    .unwrap_or(format!("func_{}", entry_point));
                instance
                    .run_by_name(&fn_name, Vec::default())
                    .unwrap_or_else(|e| {
                        panic!(
                            "{file_path:?}:{line}:{col}\nError during execution of module start routine '{fn_name}': {e}",
                        )
                    });
            }
        }
        *already_started = true;
    }

    let func = match instance.find_exported_func_idx(invoke.name) {
        Ok(f) => f,
        Err(e) => {
            panic!(
                "{}Error: {}, {:?}",
                prefix,
                e,
                &instance.wasm_module().exports
            );
        }
    };
    let function_type = instance.get_function_type_from_func_idx(func).clone();
    if invoke.args.len() != function_type.0.len() {
        panic!(
            "{}Argument number mismatch: expected {}, got {}",
            prefix,
            function_type.0.len(),
            invoke.args.len()
        );
    }
    let mut input_params = Vec::new();
    for (i, input_type) in function_type.0.iter().enumerate() {
        match &invoke.args[i] {
            &wast::WastArg::Core(WastArgCore::I32(i))
                if *input_type == ValType::Number(NumType::I32) =>
            {
                input_params.push(Value::Number(Number::I32(i.trans_u32())))
            }
            &wast::WastArg::Core(WastArgCore::I64(i))
                if *input_type == ValType::Number(NumType::I64) =>
            {
                input_params.push(Value::Number(Number::I64(i.trans_u64())))
            }
            &wast::WastArg::Core(WastArgCore::F32(f))
                if *input_type == ValType::Number(NumType::F32) =>
            {
                input_params.push(Value::Number(Number::F32(f32::from_bits(f.bits))))
            }
            &wast::WastArg::Core(WastArgCore::F64(f))
                if *input_type == ValType::Number(NumType::F64) =>
            {
                input_params.push(Value::Number(Number::F64(f64::from_bits(f.bits))))
            }
            wast::WastArg::Core(WastArgCore::V128(_)) => unimplemented!(),
            wast::WastArg::Core(WastArgCore::RefExtern(r)) => {
                input_params.push(Value::Reference(Reference::Extern(r.trans_u64() as _)))
            }
            wast::WastArg::Core(WastArgCore::RefHost(r)) => {
                input_params.push(Value::Reference(Reference::Extern(r.trans_u64() as _)))
            }
            wast::WastArg::Core(WastArgCore::RefNull(_)) => {
                input_params.push(Value::Reference(Reference::Null))
            }
            wast::WastArg::Component(_) => unimplemented!(),
            _ => panic!(
                "{}Invalid function input arg type: Expected {:?}, Got {:?}",
                prefix, input_type, invoke.args[i]
            ),
        };
    }
    instance.run_by_name(invoke.name, input_params)
}

fn parse_module(module: &mut QuoteWat) -> Result<Module, ParserError> {
    let binary_mod = module
        .encode()
        .map_err(|e| ParserError::Msg(e.to_string()))?;
    let parser = Parser::default();
    let loader = Loader::from_buf(binary_mod.clone());
    parser.parse(loader)
}

pub fn test_interpreter(file_path: &str) {
    // skip for now on master
    return;
    log::info!("Parsing spec test file: {:?}", file_path);

    let content = std::fs::read_to_string(file_path).unwrap();
    let parse_buf = wast::parser::ParseBuffer::new(&content).unwrap();
    let wast_repr: Wast = match wast::parser::parse(&parse_buf) {
        Ok(wast) => wast,
        Err(e) => {
            log::warn!(
                "Third-party wast parser failed to parse spec test file: {:?}\n{:?}",
                file_path,
                e
            );
            return;
        }
    };

    let linker: Linker = Linker::new();
    let cluster: Cluster = Cluster::new();
    let mut declared_modules: DeclaredModulesMap = HashMap::new();
    let mut linker = linker.bind_to(&cluster);

    let spectest_module_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/spectest.wasm");
    let loader = Loader::from_file(&spectest_module_path);
    let parser = Parser::default();
    let res = parser.parse(loader);
    let spectest_module = translate_module(Rc::new(res.unwrap()), &mut linker).unwrap();
    linker.transfer("spectest", spectest_module.0).unwrap();

    for directive in wast_repr.directives.into_iter() {
        let (line, col) = directive.span().linecol_in(&content);
        let line = line + 1;
        let col = col + 1;

        match directive {
            wast::WastDirective::Wat(mut module) => {
                let res = parse_module(&mut module);
                let wasm_module = match res {
                    Ok(wasm_module) => wasm_module,
                    Err(e) => {
                        panic!(
                            "Parsing failed of spec test file: {:?}:{}:{}\nError: {:?}",
                            file_path, line, col, e
                        );
                    }
                };
                let translation_result = translate_module(Rc::new(wasm_module), &mut linker)
                    .unwrap_or_else(|e| {
                        panic!(
                            "Failed to translate module: {:?}:{}:{}\nError: {:?}",
                            file_path, line, col, e
                        )
                    });
                declared_modules.insert(None, translation_result.clone());
                if let QuoteWat::Wat(Wat::Module(module)) = module {
                    declared_modules
                        .insert(module.id.map(|id| id.name().to_owned()), translation_result);
                }
            }
            wast::WastDirective::AssertMalformed {
                span: _,
                mut module,
                message,
            } => {
                if parse_module(&mut module).is_ok() {
                    panic!(
                        "expected parsing failure \"{}\" for malformed module in spec test file {:?}:{}:{}, but parsed successfully.",
                        message, file_path, line, col
                    )
                } else {
                    // TODO: This could be enabled in the future, but for now, it's not too important
                    // assert_eq!(format!("{}", res.unwrap_err()), message)
                }
            }
            wast::WastDirective::AssertInvalid {
                span: _,
                mut module,
                message,
            } => {
                let m = match parse_module(&mut module) {
                    Ok(m) => m,
                    Err(_) => return,
                };
                match translate_module(Rc::new(m), &mut linker) {
                    Ok(_) => {
                        panic!(
                            "expected translation failure \"{}\" for invalid module in spec test file {:?}:{}:{}, but translated successfully.",
                            message, file_path, line, col
                        )
                    }
                    Err(_) => return,
                }
            }
            wast::WastDirective::Register {
                span: _,
                name,
                module,
            } => {
                let instance_handle = if module.is_none() {
                    declared_modules.get(&None).unwrap().0.clone()
                } else {
                    declared_modules
                        .get(&module.map(|m| m.name().to_owned()))
                        .unwrap()
                        .0
                        .clone()
                };
                linker.transfer(name, instance_handle).unwrap_or_else(|e| {
                    panic!(
                        "Failed to register module: {:?}:{}:{}\nError: {:?}",
                        file_path, line, col, e
                    )
                });
            }
            wast::WastDirective::AssertTrap {
                span: _,
                exec,
                message,
            } => {
                match exec {
                    WastExecute::Get { module, global, .. } => {
                        let (instance, _) =
                            match declared_modules.get_mut(&module.map(|m| m.name().to_owned())) {
                                Some(m) => m,
                                None => {
                                    panic!("Module not found: {:?}", module);
                                }
                            };
                        match instance.extract_global_value_by_name(global) {
                            Some(val) => {
                                panic!(
                                    "{:?}:{}:{}\nExpected trap \"{}\", but got success: {:?}",
                                    file_path, line, col, message, val
                                );
                            }
                            None => return,
                        }
                    }
                    WastExecute::Invoke(invoke) => {
                        let res = execute_via_interpreter_backend(
                            file_path,
                            line,
                            col,
                            &invoke,
                            &mut declared_modules,
                        );
                        match res {
                            Err(RuntimeError::Trap(_)) => {
                                // assert_eq!(msg, message);
                                return;
                            }
                            Ok(v) => panic!(
                                "Expected trap \"{message}\", but got success: {v:?} @{file_path:?}:{line}:{col}",
                            ),
                            Err(e) => panic!(
                                "Expected trap \"{message}\", but got other error \"{e}\": {file_path:?}:{line}:{col}",
                            ),
                        }
                    }
                    WastExecute::Wat(m) => todo!("{:?}", m),
                }
            }
            wast::WastDirective::Invoke(invoke) => {
                execute_via_interpreter_backend(
                    file_path,
                    line,
                    col,
                    &invoke,
                    &mut declared_modules,
                )
                .unwrap();
            }
            wast::WastDirective::AssertReturn {
                span: _,
                exec: wast::WastExecute::Invoke(invoke),
                results,
            } => {
                let prefix = format!("{:?}:{}:{}\n", file_path, line, col);
                let (instance, _) =
                    match declared_modules.get(&invoke.module.map(|m| m.name().to_owned())) {
                        Some(m) => m,
                        None => {
                            panic!("{}Module not found: {:?}", prefix, invoke.module);
                        }
                    };
                let func = match instance.find_exported_func_idx(invoke.name) {
                    Ok(f) => f,
                    Err(e) => {
                        panic!(
                            "{}Error: {}, {:?}",
                            prefix,
                            e,
                            &instance.wasm_module().exports
                        );
                    }
                };
                let function_type = instance.get_function_type_from_func_idx(func).clone();
                assert_eq!(
                    results.len(),
                    function_type.1.len(),
                    "{}Result number mismatch: expected {}, got {}",
                    prefix,
                    function_type.1.len(),
                    results.len()
                );

                let res = execute_via_interpreter_backend(
                    file_path,
                    line,
                    col,
                    &invoke,
                    &mut declared_modules,
                )
                .unwrap_or_else(|e| {
                    panic!("{}Error during execution: {:?}", prefix, e);
                });
                assert_eq!(res.len(), results.len());

                for (idx, output_type) in function_type.1.iter().enumerate() {
                    check_results(&results[idx], &res[idx], output_type, &prefix);
                }
            }
            wast::WastDirective::AssertReturn {
                span: _,
                exec:
                    wast::WastExecute::Get {
                        span: _,
                        module,
                        global,
                    },
                results,
            } => {
                let (instance, _) =
                    match declared_modules.get_mut(&module.map(|m| m.name().to_owned())) {
                        Some(m) => m,
                        None => {
                            panic!("Module not found: {:?}", module);
                        }
                    };
                let val = instance
                    .extract_global_value_by_name(global)
                    .unwrap_or_else(|| {
                        panic!("Global not found: {:?}", global);
                    });
                assert_eq!(results.len(), 1);
                for expected_result in results {
                    check_results(
                        &expected_result,
                        &val,
                        &val.r#type(),
                        &format!("{:?}:{}:{}\n", file_path, line, col),
                    )
                }
            }
            wast::WastDirective::AssertException { .. } => {
                unimplemented!("AssertException");
            }
            wast::WastDirective::AssertExhaustion { .. } => {
                // TODO
                // match execute_via_llvm_backend(file_path, line, col, &call, &mut declared_modules) {
                //     Err(RuntimeError::Exhaustion) => {}
                //     Err(e) => panic!("Expected Exhaustion, Got {:?}", e),
                //     Ok(_) => panic!("Expected Exhaustion, Got Ok"),
                // };
            }
            wast::WastDirective::AssertUnlinkable { mut module, .. } => {
                let binary_mod: Vec<u8> = module.encode().unwrap_or_else(|e| {
                    panic!("Error during encoding: {:?} {file_path:?}:{line}:{col}", e);
                });
                let parser = Parser::default();
                let loader = Loader::from_buf(binary_mod.clone());
                let module = parser.parse(loader).unwrap_or_else(|e| {
                    panic!("Error during parsing: {:?} {file_path:?}:{line}:{col}", e)
                });
                let module = Rc::new(module);

                let mut engine = Engine::interpreter().unwrap_or_else(|e| {
                    panic!(
                        "Error during engine init: {:?} {file_path:?}:{line}:{col}",
                        e
                    );
                });
                engine.init(module.clone()).unwrap_or_else(|e| {
                    panic!(
                        "Error during engine init: {:?} {file_path:?}:{line}:{col}",
                        e
                    );
                });

                if linker.instantiate_and_link(module.clone(), engine).is_ok() {
                    panic!("Expected Unlinkable: {file_path:?}:{line}:{col}")
                }
            }
            wast::WastDirective::Thread(t) => {
                unimplemented!("{:?}", t);
            }
            wast::WastDirective::Wait { span, thread } => {
                unimplemented!("{:?} {:?}", span, thread);
            }
            wast::WastDirective::AssertReturn {
                span: _,
                exec: wast::WastExecute::Wat(m),
                results,
            } => {
                todo!("{:?} {:?}", m, results)
            }
        }
    }

    fn check_results(
        expected_result: &wast::WastRet,
        actual_result: &Value,
        actual_type: &ValType,
        prefix: &str,
    ) {
        match expected_result {
            &wast::WastRet::Core(WastRetCore::I32(i)) => {
                assert_eq!(*actual_type, ValType::Number(NumType::I32), "{}", prefix);
                assert_eq!(
                    *actual_result,
                    Value::Number(Number::I32(i.trans_u32())),
                    "{}",
                    prefix
                );
            }
            &wast::WastRet::Core(WastRetCore::I64(i)) => {
                assert_eq!(*actual_type, ValType::Number(NumType::I64), "{}", prefix);
                assert_eq!(
                    *actual_result,
                    Value::Number(Number::I64(i.trans_u64())),
                    "{}",
                    prefix
                );
            }
            &wast::WastRet::Core(WastRetCore::F32(NanPattern::Value(f_truth))) => {
                assert_eq!(*actual_type, ValType::Number(NumType::F32), "{}", prefix);
                let f_truth = f32::from_bits(f_truth.bits);
                let f_calculated = match *actual_result {
                    Value::Number(Number::F32(f)) => f,
                    _ => {
                        panic!("{}Expected F32 Nan, Got {:?}", prefix, actual_result);
                    }
                };
                assert_eq!(f_truth.is_nan(), f_calculated.is_nan(), "{}", prefix);
                if !f_truth.is_nan() {
                    assert_eq!(f_truth, f_calculated, "{}", prefix);
                }
            }
            &wast::WastRet::Core(WastRetCore::F64(NanPattern::Value(f_truth))) => {
                assert_eq!(*actual_type, ValType::Number(NumType::F64), "{}", prefix);
                let f_truth = f64::from_bits(f_truth.bits);
                let f_calculated = match *actual_result {
                    Value::Number(Number::F64(f)) => f,
                    _ => {
                        panic!("{}Expected F64 Nan, Got {:?}", prefix, actual_result);
                    }
                };
                assert_eq!(f_truth.is_nan(), f_calculated.is_nan(), "{}", prefix);
                if !f_truth.is_nan() {
                    assert_eq!(f_truth, f_calculated, "{}", prefix);
                }
            }
            &wast::WastRet::Core(WastRetCore::F32(NanPattern::CanonicalNan))
            | &wast::WastRet::Core(WastRetCore::F32(NanPattern::ArithmeticNan)) => {
                assert_eq!(*actual_type, ValType::Number(NumType::F32), "{}", prefix);
                match actual_result {
                    Value::Number(Number::F32(f)) => {
                        assert!(f.is_nan(), "{}", prefix);
                    }
                    _ => {
                        panic!("{}Expected F32 Nan, Got {:?}", prefix, actual_result);
                    }
                }
            }
            &wast::WastRet::Core(WastRetCore::F64(NanPattern::CanonicalNan))
            | &wast::WastRet::Core(WastRetCore::F64(NanPattern::ArithmeticNan)) => {
                assert_eq!(*actual_type, ValType::Number(NumType::F64), "{}", prefix);
                match actual_result {
                    Value::Number(Number::F64(f)) => {
                        assert!(f.is_nan(), "{}", prefix);
                    }
                    _ => {
                        panic!("{}Expected F64 Nan, Got {:?}", prefix, actual_result);
                    }
                }
            }
            wast::WastRet::Core(WastRetCore::V128(_)) => unimplemented!(),
            wast::WastRet::Core(WastRetCore::RefExtern(r)) => {
                assert_eq!(
                    *actual_type,
                    ValType::Reference(wasm_types::RefType::ExternReference),
                    "{}",
                    prefix
                );
                assert_eq!(
                    *actual_result,
                    Value::Reference(Reference::Extern(r.unwrap().trans_u64() as _)),
                    "{}",
                    prefix
                );
            }
            wast::WastRet::Core(WastRetCore::RefHost(r)) => {
                assert_eq!(
                    *actual_type,
                    ValType::Reference(wasm_types::RefType::ExternReference),
                    "{}",
                    prefix
                );
                assert_eq!(
                    *actual_result,
                    Value::Reference(Reference::Extern(r.trans_u64() as _)),
                    "{}",
                    prefix
                );
            }
            wast::WastRet::Core(WastRetCore::RefNull(_)) => {
                assert!(matches!(*actual_type, ValType::Reference(_)), "{}", prefix);
                assert_eq!(
                    *actual_result,
                    Value::Reference(Reference::Null),
                    "{}",
                    prefix
                );
            }
            wast::WastRet::Component(_) => unimplemented!(),
            _ => panic!(
                "{}Result mismatch: Expected {:?}, Got {:?}",
                prefix, actual_result, expected_result
            ),
        };
    }
}