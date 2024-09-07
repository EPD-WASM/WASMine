use clap::{Parser, Subcommand, ValueEnum};
use log::LevelFilter;
use runtime_lib::ConfigBuilder;
use simplelog::{ColorChoice, CombinedLogger, TermLogger, TerminalMode};
use std::{path::PathBuf, process::ExitCode};

#[derive(Parser, Debug)]
#[command(name = "WASMine", version = "dev")]
struct Args {
    /// Log Level
    #[arg(short, long, default_value = "warn")]
    log_level: LogLevel,

    /// Backend
    #[arg(short, long, default_value = "llvm")]
    backend: Backend,

    #[command(subcommand)]
    action: Action,
}

#[derive(Subcommand, Debug, Clone)]
enum Action {
    /// execute fully sandboxed
    Run {
        /// ".wasm" / ".cwasm" executable path
        path: PathBuf,

        /// exported Wasm function name (defaults to the set start function)
        #[arg(short, long)]
        invoke: Option<String>,

        /// wasm function arguments
        #[arg(last = true)]
        function_args: Vec<String>,
    },
    /// execute with WASI API support (CAUTION: weakens WebAssembly sandboxing)
    RunWasi {
        /// ".wasm" / ".cwasm" executable path
        path: PathBuf,

        /// exported Wasm function name (defaults to the set start function)
        #[arg(short, long)]
        invoke: Option<String>,

        /// list all directories accessible via the WASI API
        #[arg(short, long, value_delimiter = ',', value_parser = parse_wasi_dir_arg, value_name = "HOST_DIR[::GUEST_DIR]")]
        dir: Vec<(PathBuf, String)>,

        /// arguments forwarded to the WASI application
        #[arg(last = true)]
        wasi_args: Vec<String>,
    },
    /// create precompiled cwasm executable
    #[cfg(feature = "llvm")]
    Compile {
        /// `.wasm` file path
        path: PathBuf,

        /// `.cwasm` output path for the compiled executable
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

#[allow(clippy::upper_case_acronyms)]
#[derive(ValueEnum, Debug, Clone)]
enum Backend {
    #[cfg(feature = "llvm")]
    LLVM,
    #[cfg(feature = "interp")]
    Interpreter,
}

impl Action {
    fn path(&self) -> PathBuf {
        match self {
            Action::Run { path, .. } => path.clone(),
            Action::RunWasi { path, .. } => path.clone(),
            #[cfg(feature = "llvm")]
            Action::Compile { path, .. } => path.clone(),
        }
    }
}

#[derive(ValueEnum, Debug, Clone)]
enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl From<LogLevel> for LevelFilter {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Trace => LevelFilter::Trace,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Error => LevelFilter::Error,
        }
    }
}
fn parse_wasi_dir_arg(s: &str) -> Result<(PathBuf, String), String> {
    if let Some(s) = s.split_once("::") {
        Ok((PathBuf::from(s.0), s.1.to_string()))
    } else {
        Ok((PathBuf::from(s), s.to_string()))
    }
}

pub fn main() -> ExitCode {
    let args = Args::parse();

    CombinedLogger::init(vec![TermLogger::new(
        args.log_level.clone().into(),
        simplelog::Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])
    .unwrap();

    let path = args.action.path();
    let engine = match args.backend {
        #[cfg(feature = "llvm")]
        Backend::LLVM => runtime_lib::Engine::llvm().unwrap(),
        #[cfg(feature = "interp")]
        Backend::Interpreter => runtime_lib::Engine::interpreter().unwrap(),
    };

    let mut cb = ConfigBuilder::new();
    let ret = match args.action {
        Action::Run {
            invoke,
            function_args,
            ..
        } => {
            if let Some(start_func) = invoke {
                cb.set_start_function(start_func);
            }
            crate::run(&path, cb.finish(), engine, function_args)
        }
        Action::RunWasi {
            path,
            invoke,
            dir: dirs,
            mut wasi_args,
        } => {
            if let Some(start_func) = invoke {
                cb.set_start_function(start_func);
            }
            cb.set_wasi_dirs(dirs);
            // set first argument to executable path
            wasi_args.insert(0, path.to_string_lossy().to_string());
            cb.set_wasi_args(wasi_args);
            crate::run(&path, cb.finish(), engine, vec![])
        }
        #[cfg(feature = "llvm")]
        Action::Compile { output, .. } => crate::c_wasm_compilation::compile(
            &path,
            &output.unwrap_or_else(|| {
                PathBuf::new()
                    .join(path.file_name().unwrap_or_default())
                    .with_extension("cwasm")
            }),
        ),
    };
    ExitCode::from(ret)
}
