use std::env::args;
use wasm_rt::parser;

fn main() {
    let parser = parser::parser::Parser::default();
    let input = std::fs::File::open(args().nth(1).unwrap()).unwrap();
    let module = parser.parse(input).unwrap();
    log::info!("{:?}", module);
}
