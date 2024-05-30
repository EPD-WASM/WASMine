use gen_util::generate_spec_test_cases;
use parser::parser::Parser;
use wast::Wast;

pub fn test_parser(file_path: &str) {
    let content = std::fs::read_to_string(file_path).unwrap();
    let parse_buf = wast::parser::ParseBuffer::new(&content).unwrap();
    let wast_repr: Wast = match wast::parser::parse(&parse_buf) {
        Ok(wast) => wast,
        Err(e) => {
            println!(
                "Warning: Third-party wast parser failed to parse spec test file: {:?}\n{:?}",
                file_path, e
            );
            return;
        }
    };
    for directive in wast_repr.directives.into_iter() {
        let (line, col) = directive.span().linecol_in(&content);
        match directive {
            wast::WastDirective::Wat(wast::QuoteWat::Wat(wast::Wat::Module(mut module))) => {
                let binary_mod = module.encode().unwrap();
                let parser = Parser::default();
                let res = parser.parse(binary_mod.as_slice());
                if res.is_err() {
                    std::fs::write("test_module_dump.wasm", binary_mod).unwrap();
                    println!(
                        "Parsing failed of spec test file: {:?}:{}:{}\nWriting binary module to ./test_module_dump.wasm",
                        file_path, line, col
                    );
                }
                res.unwrap();
            }
            wast::WastDirective::AssertMalformed {
                span: _,
                module: wast::QuoteWat::Wat(wast::Wat::Module(mut module)),
                message,
            } => {
                let binary_mod = module.encode().unwrap();
                let parser = Parser::default();
                let res = parser.parse(binary_mod.as_slice());
                if res.is_ok() {
                    std::fs::write("test_module_dump.wasm", binary_mod).unwrap();
                    panic!(
                        "expected parsing failure \"{}\" for malformed module in spec test file {:?}:{}:{}, but parsed successfully.\nnWriting binary module to ./test_module_dump.wasm",
                        message, file_path, line, col
                    )
                }
            }
            _ => {}
        }
    }
}

generate_spec_test_cases!(test_parser);
