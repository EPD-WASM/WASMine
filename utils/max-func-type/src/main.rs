use std::io::Read;
use wasmparser::Parser;

fn main() {
    let path = std::env::args().nth(1).unwrap();
    let mut file = std::fs::File::open(path).unwrap();
    let mut content = Vec::new();
    file.read_to_end(&mut content).unwrap();
    let parser = Parser::new(0);

    let mut largest_params_cnt = 0;
    let mut largets_params = Vec::new();
    let mut largest_results_cnt = 0;
    let mut largets_results = Vec::new();

    for payload in parser.parse_all(content.as_slice()) {
        if let wasmparser::Payload::TypeSection(types) = payload.unwrap() {
            for t in types {
                for t in t.unwrap().into_types() {
                    if let wasmparser::CompositeInnerType::Func(t) = t.composite_type.inner {
                        if t.params().len() > largest_params_cnt {
                            largest_params_cnt = t.params().len();
                            largets_params = t.params().to_vec();
                        }
                        if t.results().len() > largest_results_cnt {
                            largest_results_cnt = t.results().len();
                            largets_results = t.results().to_vec();
                        }
                    }
                }
            }
        }
    }

    println!("Largest params count: {}", largest_params_cnt);
    println!("Largest params: {:?}", largets_params);
    println!("Largest results count: {}", largest_results_cnt);
    println!("Largest results: {:?}", largets_results);
}
