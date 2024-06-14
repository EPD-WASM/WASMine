use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode};
use std::env::args;
use std::io::Write;
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
    // let loader = loader::Loader::from_file(&path);
    // let parser = parser::parser::Parser::default();
    // let module = parser.parse(loader).unwrap();
    // let mut output_file = std::fs::File::create("output.ll").unwrap();
    // #[cfg(debug_assertions)]
    // write!(&mut output_file, "{}", module).unwrap();
    // #[cfg(not(debug_assertions))]
    // write!(&mut output_file, "{:?}", module).unwrap();

    let res = runtime_lib::run(path.as_os_str().to_str().unwrap());
    ExitCode::from(res)
}
