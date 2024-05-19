use log::LevelFilter;
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, TerminalMode};
use std::env::args;
use wasm_rt::parser;

fn main() {
    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])
    .unwrap();
    let parser = parser::parser::Parser::default();
    let input = std::fs::File::open(args().nth(1).unwrap()).unwrap();
    let module = parser.parse(input).unwrap();
    log::info!("{:?}", module);
}
