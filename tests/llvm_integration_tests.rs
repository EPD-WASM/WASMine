use core::panic;
use gen_util::generate_spec_test_cases;
use module::{
    objects::{
        function::Function,
        module::Module,
        value::{Number, Reference, Value},
    },
    utils::numeric_transmutes::{Bit32, Bit64},
    ModuleError,
};
use runtime_lib::{
    BoundLinker, Cluster, ClusterConfig, Engine, InstanceHandle, Linker, ResourceBuffer,
    RuntimeError,
};
use std::{collections::HashMap, path::PathBuf};
use test_log::test;
use wasm_types::ValType;
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

generate_spec_test_cases!(test_llvm, llvm);

pub fn translate_module<'a>(
    module: module::objects::module::Module,
    linker: &mut BoundLinker<'a>,
) -> Result<(InstanceHandle<'a>, bool), RuntimeError> {
    let mut engine = Engine::llvm()?;
    let module = engine.init(module)?;
    let instance_handle = linker.instantiate_and_link(module.clone(), engine)?;
    Ok((instance_handle, false))
}

fn execute_via_llvm_backend(
    file_path: &str,
    line: usize,
    col: usize,
    invoke: &wast::WastInvoke,
    declared_modules: &mut DeclaredModulesMap,
) -> Result<Vec<Value>, RuntimeError> {
    let prefix = format!("{file_path:?}:{line}:{col}\n");
    let (instance, already_started) =
        match declared_modules.get_mut(&invoke.module.map(|m| m.name().to_owned())) {
            Some(m) => m,
            None => {
                panic!("{}Module not found: {:?}", prefix, invoke.module);
            }
        };
    if !*already_started {
        if let Some(entry_point) = instance.wasm_module().meta.entry_point {
            // if the entry point is an import, we don't need to run it
            if instance.wasm_module().meta.functions[entry_point as usize]
                .get_import()
                .is_none()
            {
                let fn_name =
                    Function::query_function_name(entry_point, &instance.wasm_module().meta)
                        .map(|s| s.to_owned())
                        .unwrap_or(format!("func_{entry_point}",));
                instance
                    .get_function_by_idx(instance.find_exported_func_idx(&fn_name).unwrap()).unwrap().call(&[])
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
                &instance.wasm_module().meta.exports
            );
        }
    };
    let function_type = instance.get_function_type_from_func_idx(func);
    if invoke.args.len() != function_type.num_params() {
        panic!(
            "{}Argument number mismatch: expected {}, got {}",
            prefix,
            function_type.num_params(),
            invoke.args.len()
        );
    }
    let mut input_params = Vec::new();
    for (i, input_type) in function_type.params_iter().enumerate() {
        match &invoke.args[i] {
            &wast::WastArg::Core(WastArgCore::I32(i)) if input_type == ValType::i32() => {
                input_params.push(Value::i32(i.trans_u32()))
            }
            &wast::WastArg::Core(WastArgCore::I64(i)) if input_type == ValType::i64() => {
                input_params.push(Value::i64(i.trans_u64()))
            }
            &wast::WastArg::Core(WastArgCore::F32(f)) if input_type == ValType::f32() => {
                input_params.push(Value::f32(f32::from_bits(f.bits)))
            }
            &wast::WastArg::Core(WastArgCore::F64(f)) if input_type == ValType::f64() => {
                input_params.push(Value::f64(f64::from_bits(f.bits)))
            }
            wast::WastArg::Core(WastArgCore::V128(_)) => unimplemented!(),
            wast::WastArg::Core(WastArgCore::RefExtern(r)) => {
                input_params.push(Value::externref(r.trans_u64()))
            }
            wast::WastArg::Core(WastArgCore::RefHost(r)) => {
                input_params.push(Value::externref(r.trans_u64()))
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
    instance
        .get_function_by_idx(instance.find_exported_func_idx(invoke.name).unwrap())
        .unwrap()
        .call(&input_params)
}

fn parse_module(module: &mut QuoteWat) -> Result<Module, ModuleError> {
    let binary_mod = module
        .encode()
        .map_err(|e| ModuleError::Msg(e.to_string()))?;
    let source = ResourceBuffer::from_wasm_buf(binary_mod.clone());
    let mut module = Module::new(source);
    module.load_meta(parser::ModuleMetaLoader)?;
    module.load_meta(llvm_gen::ModuleMetaLoader)?;
    module.load_all_functions(parser::FunctionLoader)?;
    module.load_all_functions(llvm_gen::FunctionLoader)?;
    Ok(module)
}

pub fn test_llvm(file_path: &str) {
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
    let cluster: Cluster = Cluster::new(ClusterConfig::default());
    let mut declared_modules: DeclaredModulesMap = HashMap::new();
    let mut linker = linker.bind_to(&cluster);

    let spectest_module_path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/spectest.wasm");
    let source = ResourceBuffer::from_file(&spectest_module_path).unwrap();
    let mut module = Module::new(source);
    module.load_meta(parser::ModuleMetaLoader).unwrap();
    let spectest_module = translate_module(module, &mut linker).unwrap();
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
                let translation_result =
                    translate_module(wasm_module, &mut linker).unwrap_or_else(|e| {
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
                match translate_module(m, &mut linker) {
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
                        let res = execute_via_llvm_backend(
                            file_path,
                            line,
                            col,
                            &invoke,
                            &mut declared_modules,
                        );
                        match res {
                            Err(_) => {
                                // assert_eq!(msg, message);
                                return;
                            }
                            Ok(v) => panic!(
                                "Expected trap \"{message}\", but got success: {v:?} @{file_path:?}:{line}:{col}",
                            ),
                        }
                    }
                    WastExecute::Wat(m) => todo!("{:?}", m),
                }
            }
            wast::WastDirective::Invoke(invoke) => {
                execute_via_llvm_backend(file_path, line, col, &invoke, &mut declared_modules)
                    .unwrap();
            }
            wast::WastDirective::AssertReturn {
                span: _,
                exec: wast::WastExecute::Invoke(invoke),
                results,
            } => {
                let prefix = format!("{file_path:?}:{line}:{col}\n");
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
                            &instance.wasm_module().meta.exports
                        );
                    }
                };
                let function_type = instance.get_function_type_from_func_idx(func);
                assert_eq!(
                    results.len(),
                    function_type.num_results(),
                    "{}Result number mismatch: expected {}, got {}",
                    prefix,
                    function_type.num_results(),
                    results.len()
                );

                let res =
                    execute_via_llvm_backend(file_path, line, col, &invoke, &mut declared_modules)
                        .unwrap_or_else(|e| {
                            panic!("{}Error during execution: {:?}", prefix, e);
                        });
                assert_eq!(res.len(), results.len());

                for (idx, output_type) in function_type.results_iter().enumerate() {
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
                        val.r#type(),
                        &format!("{file_path:?}:{line}:{col}\n"),
                    )
                }
            }
            wast::WastDirective::AssertException { .. } => {
                unimplemented!("AssertException");
            }
            wast::WastDirective::AssertExhaustion { call, .. } => {
                match execute_via_llvm_backend(file_path, line, col, &call, &mut declared_modules) {
                    Err(RuntimeError::Exhaustion) => {}
                    Err(e) => panic!("Expected Exhaustion, Got {:?}", e),
                    Ok(_) => panic!("Expected Exhaustion, Got Ok"),
                };
            }
            wast::WastDirective::AssertUnlinkable { mut module, .. } => {
                let binary_mod: Vec<u8> = module.encode().unwrap_or_else(|e| {
                    panic!("Error during encoding: {:?} {file_path:?}:{line}:{col}", e);
                });
                let source = ResourceBuffer::from_wasm_buf(binary_mod.clone());
                let mut module = Module::new(source);
                module
                    .load_meta(parser::ModuleMetaLoader)
                    .unwrap_or_else(|e| {
                        panic!("Error during parsing: {:?} {file_path:?}:{line}:{col}", e)
                    });

                let mut engine = Engine::llvm().unwrap_or_else(|e| {
                    panic!(
                        "Error during engine init: {:?} {file_path:?}:{line}:{col}",
                        e
                    );
                });
                let module = engine.init(module).unwrap_or_else(|e| {
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
        actual_type: ValType,
        prefix: &str,
    ) {
        match expected_result {
            &wast::WastRet::Core(WastRetCore::I32(i)) => {
                assert_eq!(actual_type, ValType::i32(), "{prefix}");
                assert_eq!(*actual_result, Value::i32(i.trans_u32()), "{prefix}");
            }
            &wast::WastRet::Core(WastRetCore::I64(i)) => {
                assert_eq!(actual_type, ValType::i64(), "{prefix}");
                assert_eq!(*actual_result, Value::i64(i.trans_u64()), "{prefix}");
            }
            &wast::WastRet::Core(WastRetCore::F32(NanPattern::Value(f_truth))) => {
                assert_eq!(actual_type, ValType::f32(), "{prefix}");
                let f_truth = f32::from_bits(f_truth.bits);
                let f_calculated = match *actual_result {
                    Value::Number(Number::F32(f)) => f,
                    _ => {
                        panic!("{}Expected F32 Nan, Got {:?}", prefix, actual_result);
                    }
                };
                assert_eq!(f_truth.is_nan(), f_calculated.is_nan(), "{prefix}");
                if !f_truth.is_nan() {
                    assert_eq!(f_truth, f_calculated, "{prefix}");
                }
            }
            &wast::WastRet::Core(WastRetCore::F64(NanPattern::Value(f_truth))) => {
                assert_eq!(actual_type, ValType::f64(), "{prefix}");
                let f_truth = f64::from_bits(f_truth.bits);
                let f_calculated = match *actual_result {
                    Value::Number(Number::F64(f)) => f,
                    _ => {
                        panic!("{}Expected F64 Nan, Got {:?}", prefix, actual_result);
                    }
                };
                assert_eq!(f_truth.is_nan(), f_calculated.is_nan(), "{prefix}");
                if !f_truth.is_nan() {
                    assert_eq!(f_truth, f_calculated, "{prefix}");
                }
            }
            &wast::WastRet::Core(WastRetCore::F32(NanPattern::CanonicalNan))
            | &wast::WastRet::Core(WastRetCore::F32(NanPattern::ArithmeticNan)) => {
                assert_eq!(actual_type, ValType::f32(), "{prefix}");
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
                assert_eq!(actual_type, ValType::f64(), "{prefix}");
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
                    actual_type,
                    ValType::Reference(wasm_types::RefType::ExternReference),
                    "{prefix}"
                );
                assert_eq!(
                    *actual_result,
                    Value::externref(r.unwrap().trans_u64()),
                    "{prefix}"
                );
            }
            wast::WastRet::Core(WastRetCore::RefHost(r)) => {
                assert_eq!(
                    actual_type,
                    ValType::Reference(wasm_types::RefType::ExternReference),
                    "{prefix}"
                );
                assert_eq!(*actual_result, Value::externref(r.trans_u64()), "{prefix}");
            }
            wast::WastRet::Core(WastRetCore::RefNull(_)) => {
                assert!(matches!(actual_type, ValType::Reference(_)), "{}", prefix);
                assert_eq!(
                    *actual_result,
                    Value::Reference(Reference::Null),
                    "{prefix}"
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
