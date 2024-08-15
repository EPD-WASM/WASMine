use crate::config::{Config, ConfigBuilder};
use clap::{Parser, ValueEnum};
use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, TermLogger, TerminalMode};
use std::{path::PathBuf, process::ExitCode};

#[derive(Parser, Debug)]
#[command(name = "WASMine", version = "dev")]
struct Args {
    /// ".wasm" executable path
    path: PathBuf,

    /// exported Wasm function name (defaults to the set start function)
    #[arg(short, long)]
    invoke: Option<String>,

    /// enable WASI API support (CAUTION: weakens WebAssembly sandboxing)
    #[arg(short, long, default_value_t = false)]
    wasi: bool,

    /// (WASI-only) list all directories accessible via the WASI API
    #[arg(short, long, requires = "wasi", value_delimiter = ',', value_parser = parse_wasi_dir_arg, value_name = "HOST_DIR[::GUEST_DIR]")]
    dir: Vec<(PathBuf, String)>,

    /// (WASI-only) additional arguments for the WASI application
    #[arg(last = true, requires = "wasi")]
    args: Vec<String>,

    /// Log Level
    #[arg(short, long, default_value = "warn")]
    log_level: LogLevel,
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

impl From<Args> for Config {
    fn from(mut args: Args) -> Self {
        let mut cb = ConfigBuilder::new()
            .enable_wasi(args.wasi)
            .set_wasi_dirs(args.dir);
        if args.wasi {
            // set first argument to executable path
            let mut wasi_args = vec![args.path.to_string_lossy().to_string()];
            wasi_args.append(&mut args.args);
            cb = cb.set_wasi_args(wasi_args);
        }
        if let Some(start_func) = args.invoke {
            cb = cb.set_start_function(start_func);
        }
        cb.finish()
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

    let path: PathBuf = args.path.clone();
    let config: Config = args.into();
    crate::run(&path, config);
    ExitCode::SUCCESS
}
