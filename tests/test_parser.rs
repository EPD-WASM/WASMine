use module::ModuleError;
use parser::Parser;
use runtime_lib::FunctionLoaderInterface;
use wast::Wast;

pub fn test_parser(file_path: &str) {
    let content = std::fs::read_to_string(file_path).unwrap();
    let parse_buf = wast::parser::ParseBuffer::new(&content).unwrap();
    let wast_repr: Wast = match wast::parser::parse(&parse_buf) {
        Ok(wast) => wast,
        Err(e) => {
            eprintln!(
                "Warning: Third-party wast parser failed to parse spec test file: {file_path:?}\n{e:?}"
            );
            return;
        }
    };
    for directive in wast_repr.directives.into_iter() {
        let (line, col) = directive.span().linecol_in(&content);
        match directive {
            wast::WastDirective::Wat(wast::QuoteWat::Wat(wast::Wat::Module(mut module))) => {
                let binary_mod = module.encode().unwrap();
                let module = Parser::parse_from_buf(binary_mod.clone());
                if module.is_err() {
                    std::fs::write("test_module_dump.wasm", binary_mod).unwrap();
                    eprintln!(
                        "Parsing failed of spec test file: {file_path:?}:{line}:{col}\nWriting binary module to ./test_module_dump.wasm"
                    );
                }
                let module = module.unwrap();
                if parser::FunctionLoader.parse_all_functions(&module).is_err() {
                    eprintln!("Failed to parse IR functions {file_path:?}:{line}:{col}");
                }
            }
            wast::WastDirective::AssertMalformed {
                span: _,
                module: wast::QuoteWat::Wat(wast::Wat::Module(mut module)),
                message,
            } => {
                let binary_mod = module.encode().unwrap();
                let module = Parser::parse_from_buf(binary_mod.clone())
                    .map_err(|e| ModuleError::Msg(e.to_string()))
                    .and_then(|m| parser::FunctionLoader.parse_all_functions(&m));
                if module.is_ok() {
                    std::fs::write("test_module_dump.wasm", binary_mod).unwrap();
                    panic!(
                        "expected parsing failure \"{message}\" for malformed module in spec test file {file_path:?}:{line}:{col}, but parsed successfully.\nnWriting binary module to ./test_module_dump.wasm"
                    )
                }
            }
            _ => {}
        }
    }
}
