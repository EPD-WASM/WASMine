use iai_callgrind::{library_benchmark, library_benchmark_group, main, LibraryBenchmarkConfig};
use ir::structs::value::{Number, Value};
use loader::Loader;
use std::{path::PathBuf, rc::Rc};

#[library_benchmark]
#[bench::multiple("fibonacci.wasm")]
pub fn wasmine_llvm_e2e_iai(path: &str) {
    let wasm_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(path);

    let wasm_bytes = std::fs::read(wasm_file_path).unwrap();
    let wasmine_module = Rc::new(
        parser::parser::Parser::default()
            .parse(Loader::from_buf(wasm_bytes.clone()))
            .unwrap(),
    );
    let wasmine_cluster = runtime_lib::Cluster::new();
    let mut wasmine_engine = runtime_lib::Engine::llvm().unwrap();
    wasmine_engine.init(wasmine_module.clone()).unwrap();

    let mut wasmine_instance = runtime_lib::BoundLinker::new(&wasmine_cluster)
        .instantiate_and_link(wasmine_module.clone(), wasmine_engine)
        .unwrap();

    wasmine_instance
        .run_by_name("_start", vec![Value::Number(Number::I32(5))])
        .unwrap();
}

library_benchmark_group!(
    name = bench_fibonacci_group;
    config = LibraryBenchmarkConfig::default().raw_callgrind_args(["--cache-sim=no"]);
    benchmarks = wasmine_llvm_e2e_iai
);

main!(library_benchmark_groups = bench_fibonacci_group);
