extern crate anyhow;
extern crate runtime_lib;

use anyhow::{Context, Result};
use runtime_lib::{
    BoundLinker, Cluster, Engine, InstanceHandle, Linker, Loader, Parser, RuntimeError, WasmModule,
};
use std::rc::Rc;
use std::slice;
use std::{ffi::c_void, process::ExitCode};

#[repr(C)]
pub struct WasmBenchConfig {
    /// The working directory where benchmarks should be executed.
    pub working_dir_ptr: *const u8,
    pub working_dir_len: usize,

    /// The file path that should be created and used as `stdout`.
    pub stdout_path_ptr: *const u8,
    pub stdout_path_len: usize,

    /// The file path that should be created and used as `stderr`.
    pub stderr_path_ptr: *const u8,
    pub stderr_path_len: usize,

    /// The (optional) file path that should be opened and used as `stdin`. If
    /// not provided, then the WASI context will not have a `stdin` initialized.
    pub stdin_path_ptr: *const u8,
    pub stdin_path_len: usize,

    /// The functions to start and stop performance timers/counters during Wasm
    /// compilation.
    pub compilation_timer: *mut u8,
    pub compilation_start: extern "C" fn(*mut u8),
    pub compilation_end: extern "C" fn(*mut u8),

    /// The functions to start and stop performance timers/counters during Wasm
    /// instantiation.
    pub instantiation_timer: *mut u8,
    pub instantiation_start: extern "C" fn(*mut u8),
    pub instantiation_end: extern "C" fn(*mut u8),

    /// The functions to start and stop performance timers/counters during Wasm
    /// execution.
    pub execution_timer: *mut u8,
    pub execution_start: extern "C" fn(*mut u8),
    pub execution_end: extern "C" fn(*mut u8),

    /// The (optional) flags to use when running Wasmtime. These correspond to
    /// the flags used when running Wasmtime from the command line.
    pub execution_flags_ptr: *const u8,
    pub execution_flags_len: usize,
}

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn wasm_bench_create(
    config: WasmBenchConfig,
    out_bench_ptr: *mut *mut c_void,
) -> ExitCode {
    // let working_dir = config.working_dir().unwrap();
    // let stdout_path = config.stdout_path().unwrap();
    // let stderr_path = config.stderr_path().unwrap();
    // let stdin_path = config.stdin_path().unwrap();

    let state = Box::new(
        BenchState::new(
            config.compilation_timer,
            config.compilation_start,
            config.compilation_end,
            config.instantiation_timer,
            config.instantiation_start,
            config.instantiation_end,
            config.execution_timer,
            config.execution_start,
            config.execution_end,
        )
        .unwrap(),
    );

    assert!(!out_bench_ptr.is_null());
    unsafe {
        *out_bench_ptr = Box::into_raw(state) as *mut c_void;
    }
    to_exit_code(Ok(()))
}

/// Free the engine state allocated by this library.
#[no_mangle]
pub extern "C" fn wasm_bench_free(state: *mut c_void) {
    assert!(!state.is_null());
    unsafe {
        drop(Box::from_raw(state as *mut BenchState));
    }
}

/// Compile the Wasm benchmark module.
#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn wasm_bench_compile(
    state: *mut c_void,
    wasm_bytes: *const u8,
    wasm_bytes_length: usize,
) -> ExitCode {
    let state = unsafe { (state as *mut BenchState).as_mut().unwrap() };
    let wasm_bytes = unsafe { slice::from_raw_parts(wasm_bytes, wasm_bytes_length) };
    let result = state.compile(wasm_bytes).context("failed to compile");
    to_exit_code(result)
}

/// Instantiate the Wasm benchmark module.
#[no_mangle]
pub extern "C" fn wasm_bench_instantiate(state: *mut c_void) -> ExitCode {
    let state = unsafe { (state as *mut BenchState).as_mut().unwrap() };
    let result = state.instantiate().context("failed to instantiate");
    to_exit_code(result)
}

/// Execute the Wasm benchmark module.
#[no_mangle]
pub extern "C" fn wasm_bench_execute(state: *mut c_void) -> ExitCode {
    let state = unsafe { (state as *mut BenchState).as_mut().unwrap() };
    let result = state.execute().context("failed to execute");
    to_exit_code(result)
}

fn to_exit_code<T>(result: impl Into<Result<T>>) -> ExitCode {
    match result.into() {
        Ok(_) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{:?}", error);
            ExitCode::FAILURE
        }
    }
}

/// This structure contains the actual Rust implementation of the state required
/// to manage the Wasmtime engine between calls.
struct BenchState<'a> {
    linker: BoundLinker<'a>,
    module: Option<Rc<WasmModule>>,
    instance: Option<InstanceHandle<'a>>,

    compilation_timer: *mut u8,
    compilation_start: extern "C" fn(*mut u8),
    compilation_end: extern "C" fn(*mut u8),
    instantiation_timer: *mut u8,
    instantiation_start: extern "C" fn(*mut u8),
    instantiation_end: extern "C" fn(*mut u8),
    _execution_timer: *mut u8,
    _execution_start: extern "C" fn(*mut u8),
    _execution_end: extern "C" fn(*mut u8),

    _cluster: Cluster,
}

impl<'a> BenchState<'a> {
    #[allow(clippy::too_many_arguments)]
    fn new(
        compilation_timer: *mut u8,
        compilation_start: extern "C" fn(*mut u8),
        compilation_end: extern "C" fn(*mut u8),
        instantiation_timer: *mut u8,
        instantiation_start: extern "C" fn(*mut u8),
        instantiation_end: extern "C" fn(*mut u8),
        execution_timer: *mut u8,
        execution_start: extern "C" fn(*mut u8),
        execution_end: extern "C" fn(*mut u8),
    ) -> Result<Self> {
        let mut linker = Linker::new();
        let cluster = Cluster::new();

        let exec_start_fun = move || {
            execution_start(execution_timer);
        };
        linker.link_host_function(
            "bench",
            "start",
            Box::into_raw(Box::new(exec_start_fun)) as *const c_void,
        );

        let exec_end_fun = move || {
            execution_end(execution_timer);
        };
        linker.link_host_function(
            "bench",
            "end",
            Box::into_raw(Box::new(exec_end_fun)) as *const c_void,
        );

        // we need to "ausdribblen" the lifetime checker, because we build a self-referencial struct
        // still safe, because we deallocate the cluster last (last struct member)
        let unsafe_cluster_ref = unsafe { &*(&cluster as *const Cluster) };
        let linker = linker.bind_to(unsafe_cluster_ref);

        Ok(Self {
            linker,
            module: None,
            instance: None,

            compilation_timer,
            compilation_start,
            compilation_end,
            instantiation_timer,
            instantiation_start,
            instantiation_end,

            _execution_timer: execution_timer,
            _execution_start: execution_start,
            _execution_end: execution_end,

            _cluster: cluster,
        })
    }

    fn compile(&mut self, bytes: &[u8]) -> Result<()> {
        assert!(
            self.module.is_none(),
            "create a new engine to repeat compilation"
        );

        (self.compilation_start)(self.compilation_timer);
        let loader = Loader::from_buf(bytes.to_vec());
        let parser = Parser::default();
        let module = parser.parse(loader)?;
        (self.compilation_end)(self.compilation_timer);

        self.module = Some(Rc::new(module));
        Ok(())
    }

    fn instantiate(&mut self) -> Result<()> {
        let module = self
            .module
            .as_ref()
            .expect("compile the module before instantiating it");

        (self.instantiation_start)(self.instantiation_timer);
        let instance = self
            .linker
            .instantiate_and_link(module.clone(), Engine::llvm()?)?;
        (self.instantiation_end)(self.instantiation_timer);

        self.instance = Some(instance);
        Ok(())
    }

    fn execute(&mut self) -> Result<(), RuntimeError> {
        let mut instance = self
            .instance
            .take()
            .expect("instantiate the module before executing it");

        match instance.run_by_name("_start", Vec::new()) {
            Ok(_) => Ok(()),
            Err(trap) => Err(trap),
        }
    }
}
