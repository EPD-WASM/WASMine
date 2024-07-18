use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode};
use std::env::args;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Debug,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])
    .unwrap();
    let path = PathBuf::from(args().nth(1).unwrap());
    let res = runtime_lib::run(path.as_os_str().to_str().unwrap());
    ExitCode::from(res)
}
