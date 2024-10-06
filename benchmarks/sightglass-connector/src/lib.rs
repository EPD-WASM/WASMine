extern crate anyhow;
extern crate runtime_lib;
use anyhow::{Context, Result};
use llvm_gen::Translator;
use runtime_lib::{
    Cluster, ClusterConfig, Engine, InstanceHandle, Linker, Parser, RuntimeError, WasmModule,
};
use std::ffi::c_void;
use std::os::fd::IntoRawFd;
use std::path::PathBuf;
use std::pin::Pin;
use std::rc::Rc;
use std::slice;
use wasi::{PreopenDirInheritPerms, PreopenDirPerms, WasiContext, WasiContextBuilder};

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

impl WasmBenchConfig {
    fn working_dir(&self) -> Result<PathBuf> {
        let working_dir =
            unsafe { std::slice::from_raw_parts(self.working_dir_ptr, self.working_dir_len) };
        let working_dir = std::str::from_utf8(working_dir)
            .context("given working directory is not valid UTF-8")?;
        Ok(working_dir.into())
    }

    fn stdout_path(&self) -> Result<PathBuf> {
        let stdout_path =
            unsafe { std::slice::from_raw_parts(self.stdout_path_ptr, self.stdout_path_len) };
        let stdout_path =
            std::str::from_utf8(stdout_path).context("given stdout path is not valid UTF-8")?;
        Ok(stdout_path.into())
    }

    fn stderr_path(&self) -> Result<PathBuf> {
        let stderr_path =
            unsafe { std::slice::from_raw_parts(self.stderr_path_ptr, self.stderr_path_len) };
        let stderr_path =
            std::str::from_utf8(stderr_path).context("given stderr path is not valid UTF-8")?;
        Ok(stderr_path.into())
    }

    fn stdin_path(&self) -> Result<Option<PathBuf>> {
        if self.stdin_path_ptr.is_null() {
            return Ok(None);
        }

        let stdin_path =
            unsafe { std::slice::from_raw_parts(self.stdin_path_ptr, self.stdin_path_len) };
        let stdin_path =
            std::str::from_utf8(stdin_path).context("given stdin path is not valid UTF-8")?;
        Ok(Some(stdin_path.into()))
    }
}

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn wasm_bench_create(
    config: WasmBenchConfig,
    out_bench_ptr: *mut *mut c_void,
) -> u8 {
    let working_dir = config.working_dir().unwrap();
    let stdout_path = config.stdout_path().unwrap();
    let stderr_path = config.stderr_path().unwrap();
    let stdin_path = config.stdin_path().unwrap();

    let make_wasi_ctxt = move || {
        let mut builder = WasiContextBuilder::new();

        let stdout = std::fs::File::create(&stdout_path)
            .unwrap_or_else(|_| panic!("failed to create {}", stdout_path.display()));
        builder.set_stdout(stdout.into_raw_fd(), true);

        let stderr = std::fs::File::create(&stderr_path)
            .unwrap_or_else(|_| panic!("failed to create {}", stderr_path.display()));
        builder.set_stderr(stderr.into_raw_fd(), true);

        if let Some(stdin_path) = &stdin_path {
            let stdin = std::fs::File::open(stdin_path)
                .unwrap_or_else(|_| panic!("failed to open {}", stdin_path.display()));
            builder.set_stdin(stdin.into_raw_fd(), true);
        }

        // Allow access to the working directory so that the benchmark can read
        // its input workload(s).
        builder.preopen_dir(
            working_dir.clone(),
            ".",
            PreopenDirPerms::all(),
            PreopenDirInheritPerms::all(),
        )?;

        // Pass this env var along so that the benchmark program can use smaller
        // input workload(s) if it has them and that has been requested.
        if let Ok(val) = std::env::var("WASM_BENCH_USE_SMALL_WORKLOAD") {
            builder.env("WASM_BENCH_USE_SMALL_WORKLOAD".to_string(), val);
        }

        Ok(builder.finish())
    };

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
            make_wasi_ctxt,
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
) -> u8 {
    let state = unsafe { (state as *mut BenchState).as_mut().unwrap() };
    let wasm_bytes = unsafe { slice::from_raw_parts(wasm_bytes, wasm_bytes_length) };
    let result = state.compile(wasm_bytes).context("failed to compile");
    to_exit_code(result)
}

/// Instantiate the Wasm benchmark module.
#[no_mangle]
pub extern "C" fn wasm_bench_instantiate(state: *mut c_void) -> u8 {
    let state = unsafe { (state as *mut BenchState).as_mut().unwrap() };
    let result = state.instantiate().context("failed to instantiate");
    to_exit_code(result)
}

/// Execute the Wasm benchmark module.
#[no_mangle]
pub extern "C" fn wasm_bench_execute(state: *mut c_void) -> u8 {
    let state = unsafe { (state as *mut BenchState).as_mut().unwrap() };
    let result = state.execute().context("failed to execute");
    to_exit_code(result)
}

fn to_exit_code<T>(result: impl Into<Result<T>>) -> u8 {
    match result.into() {
        Ok(_) => 0,
        Err(error) => {
            eprintln!("{:?}", error);
            1
        }
    }
}

/// This structure contains the actual Rust implementation of the state required
/// to manage the Wasmtime engine between calls.
struct BenchState<'a> {
    linker: Linker,
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

    cluster: Pin<Box<Cluster>>,

    make_wasi_cx: Box<dyn FnMut() -> Result<WasiContext, RuntimeError>>,
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
        make_wasi_cx: impl FnMut() -> Result<WasiContext, RuntimeError> + 'static,
    ) -> Result<Self> {
        let mut linker = Linker::new();
        let cluster = Cluster::new(ClusterConfig::default());

        linker.link_host_function("bench", "start", move || {
            execution_start(execution_timer);
        });
        linker.link_host_function("bench", "end", move || {
            execution_end(execution_timer);
        });

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

            cluster: Box::pin(cluster),

            make_wasi_cx: Box::new(make_wasi_cx),
        })
    }

    fn compile(&mut self, bytes: &[u8]) -> Result<()> {
        assert!(
            self.module.is_none(),
            "create a new engine to repeat compilation"
        );

        (self.compilation_start)(self.compilation_timer);
        let module = Parser::parse_from_buf(bytes.to_vec()).unwrap();
        Translator::translate_module_meta(&module).unwrap();
        module.load_all_functions(llvm_gen::FunctionLoader).unwrap();
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
        let wasi_ctxt = (self.make_wasi_cx)()?;
        let engine = Engine::llvm()?;
        // TODO: this is broken
        // let module = engine.init(module)?;

        let instance = self
            .linker
            .bind_to(unsafe { &*(&*self.cluster.as_ref() as *const _) })
            .instantiate_and_link_with_wasi(module.clone(), engine, wasi_ctxt)?;
        (self.instantiation_end)(self.instantiation_timer);

        self.instance = Some(instance);
        Ok(())
    }

    fn execute(&mut self) -> Result<(), RuntimeError> {
        let instance = self
            .instance
            .take()
            .expect("instantiate the module before executing it");

        let func = instance.get_function_by_idx(instance.query_start_function()?)?;
        match func.call(&[]) {
            Ok(_) => Ok(()),
            Err(trap) => Err(trap),
        }
    }
}
