use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput};
use ir::structs::value::{Number, Value};
use loader::Loader;
use std::{path::PathBuf, rc::Rc};

const EXACT_ITER_UNTIL: u32 = 30;

pub fn wasmine_interp_fibonacci(c: &mut Criterion) {
    let wasm_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("fibonacci.wasm");

    let mut group = c.benchmark_group("fibonacci");
    group.throughput(Throughput::Elements(1));
    group.sample_size(10);
    group.sampling_mode(criterion::SamplingMode::Flat);

    let wasm_bytes = std::fs::read(wasm_file_path).unwrap();

    for num in (0..EXACT_ITER_UNTIL).chain((EXACT_ITER_UNTIL..=40).step_by(5)) {
        if num > 30 {
            // Skip the test for the interpreter engine for large numbers
            continue;
        }
        group.bench_with_input(BenchmarkId::new("wasmine_interp", num), &num, |b, num| {
            b.iter_batched(
                || wasm_bytes.clone(),
                |wasm_bytes| {
                    let wasmine_module = Rc::new(
                        parser::parser::Parser::default()
                            .parse(Loader::from_buf(wasm_bytes))
                            .unwrap(),
                    );
                    let wasmine_cluster = runtime_lib::Cluster::new();
                    let mut wasmine_engine = runtime_lib::Engine::interpreter().unwrap();
                    wasmine_engine.init(wasmine_module.clone()).unwrap();

                    let mut wasmine_instance = runtime_lib::BoundLinker::new(&wasmine_cluster)
                        .instantiate_and_link(wasmine_module.clone(), wasmine_engine)
                        .unwrap();

                    wasmine_instance
                        .run_by_name("_start", vec![Value::Number(Number::I32(*num))])
                        .unwrap()
                },
                BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = wasmine_interp_fibonacci
);
criterion_main!(benches);
