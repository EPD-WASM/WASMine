use crate::{InterpreterContext, InterpreterError, InterpreterFunc, StackFrame, VariableStore};
use module::{
    basic_block::BasicBlockID,
    instructions::{FunctionIR, VariableID},
    objects::{
        function::{self, FunctionImport, FunctionSource},
        import::Import,
        value::ValueRaw,
    },
    InstructionDecoder,
};
use runtime_interface::ExecutionContext;
use wasm_types::{FuncIdx, ImportDesc};

pub(super) fn break_util(ctx: &mut InterpreterContext, target: BasicBlockID) {
    let function_idx = ctx.stack.last_mut().unwrap().fn_idx;
    let current_func = {
        let ir: &Vec<FunctionIR> = &ctx.ir;

        let fn_meta = match (&ctx).module.meta.functions.get(function_idx as usize) {
            Some(meta) => meta,
            None => unreachable!(),
        };

        match &fn_meta.source {
            FunctionSource::Import(FunctionImport { import_idx }) => {
                InterpreterFunc::Import(*import_idx)
            }
            FunctionSource::Wasm(_) => InterpreterFunc::IR(&ir[function_idx as usize]),
        }
    };
    let stack_frame = ctx.stack.last_mut().unwrap();
    let last_bb_idx = stack_frame.bb_id;

    stack_frame.bb_id = target;
    stack_frame.last_bb_id = last_bb_idx;

    let currernt_func_ir = match current_func {
        InterpreterFunc::IR(function_ir) => function_ir,
        InterpreterFunc::Import(_) => unreachable!(),
    };

    let basic_block = currernt_func_ir
        .bbs
        .iter()
        .find(|bb| bb.id == target)
        .unwrap();

    let instrs = basic_block.instructions.clone(); //slow

    stack_frame.decoder = InstructionDecoder::new(instrs);
}

pub(crate) fn call_util(
    ctx: &mut InterpreterContext,
    func_idx: FuncIdx,
    call_params: &[VariableID],
    return_bb: BasicBlockID,
    return_vars: &[VariableID],
) -> Option<Vec<ValueRaw>> {
    // not the best design but only imported functions can and must return values here.
    let func = &ctx.module.meta.functions[func_idx as usize];

    log::trace!("Calling function: {:#?}", func);

    let func = {
        let ir: &Vec<FunctionIR> = &ctx.ir;

        let fn_meta = match (&ctx).module.meta.functions.get(func_idx as usize) {
            Some(meta) => meta,
            None => unreachable!(),
        };

        match &fn_meta.source {
            FunctionSource::Import(FunctionImport { import_idx }) => {
                InterpreterFunc::Import(*import_idx)
            }
            FunctionSource::Wasm(_) => InterpreterFunc::IR(&ir[func_idx as usize]),
        }
    };

    match func {
        InterpreterFunc::Import(import_idx) => {
            let mut ret =
                unsafe { call_import_util(ctx, import_idx as usize, call_params, return_vars) };

            let mut stack_frame = ctx.stack.last_mut().unwrap();
            stack_frame.was_imported_terminator = true;

            Some(ret)
        }
        InterpreterFunc::IR(f_int) => {
            ctx.exec_ctx.recursion_size += 1;
            let stack_frame = ctx.stack.last_mut().unwrap();
            let bbs = &f_int.bbs;

            let mut new_stack_frame = StackFrame {
                fn_idx: func_idx,
                fn_local_vars: VariableStore::new(Vec::new()),
                bb_id: bbs[0].id,
                last_bb_id: 0,
                return_vars: Vec::from(return_vars),
                decoder: InstructionDecoder::new(bbs[0].instructions.clone()),
                vars: VariableStore::new(Vec::new()),
                was_imported_terminator: false,
            };

            for (idx, &param) in call_params.iter().enumerate() {
                let var = stack_frame.vars.get(param);
                new_stack_frame.fn_local_vars.set(idx, var);
            }

            stack_frame.last_bb_id = stack_frame.bb_id;
            stack_frame.bb_id = return_bb;

            ctx.stack.push(new_stack_frame);
            None
        }
    }
}

pub(crate) unsafe fn call_import_helper(
    ctx: &mut InterpreterContext,
    import_idx: usize,
    call_params: &[ValueRaw],
) -> Vec<ValueRaw> {
    log::debug!("calling import with idx: {}", import_idx);
    let import = &ctx.module.meta.imports[import_idx];
    let import_name = &import.name;
    let module_name = &import.module;
    log::debug!("import name: {:?}", import_name);
    let fn_name = format!("__import__{}.{}__", module_name, import_name);
    let ctx_name = format!("__import_ctxt__{}.{}__", module_name, import_name);
    log::debug!("fn name: {:?}, ctx name: {:?}", &fn_name, &ctx_name);
    let fn_type: wasm_types::FuncType = if let ImportDesc::Func(type_index) = &import.desc {
        ctx.module.meta.function_types[*type_index as usize].clone()
    } else {
        panic!("Import is not a function");
    };
    let func_ptr = ctx
        .imported_symbols
        .get(&fn_name)
        .expect(
            format!(
                "Function {} not found. Available symbols:\n{:#?}",
                fn_name,
                ctx.imported_symbols.keys()
            )
            .as_str(),
        )
        .as_ptr();

    let ctx_ptr = ctx
        .imported_symbols
        .get(&ctx_name)
        .expect(
            format!(
                "Context {} not found. Available symbols:\n{:#?}",
                ctx_name,
                ctx.imported_symbols.keys()
            )
            .as_str(),
        )
        .as_ptr();

    let func: fn(*mut ExecutionContext, *const ValueRaw, *mut ValueRaw) =
        std::mem::transmute(func_ptr);

    let fn_ctx: *mut ExecutionContext = std::mem::transmute(ctx_ptr);

    let mut ret_values: Vec<ValueRaw> = vec![0.into(); fn_type.num_results()];

    func(fn_ctx, call_params.as_ptr(), ret_values.as_mut_ptr());

    ret_values
}

pub(crate) unsafe fn call_import_util(
    ctx: &mut InterpreterContext,
    import_idx: usize,
    call_params: &[usize],
    return_vars: &[usize],
) -> Vec<ValueRaw> {
    let stack_frame = ctx.stack.last_mut().unwrap();
    let mut params = Vec::new();
    for &param in call_params {
        let var = stack_frame.vars.get(param);
        params.push(ValueRaw::from(var));
    }

    let ret_values = call_import_helper(ctx, import_idx, &params);

    let stack_frame = ctx.stack.last_mut().unwrap();
    for (idx, &var) in return_vars.iter().enumerate() {
        stack_frame.vars.set(var, ret_values[idx]);
    }

    ret_values
}
