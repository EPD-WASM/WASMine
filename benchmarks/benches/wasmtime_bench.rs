use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput};
use std::path::PathBuf;

const EXACT_ITER_UNTIL: u32 = 30;

pub fn wasmtime_fibonacci(c: &mut Criterion) {
    let wasm_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("fibonacci.wasm");

    let mut group = c.benchmark_group("fibonacci");
    group.throughput(Throughput::Elements(1));
    group.sample_size(10);
    group.sampling_mode(criterion::SamplingMode::Flat);

    let wasm_bytes = std::fs::read(wasm_file_path).unwrap();

    for num in (0..EXACT_ITER_UNTIL).chain((EXACT_ITER_UNTIL..=40).step_by(5)) {
        group.bench_with_input(BenchmarkId::new("wasmtime", num), &num, |b, num| {
            b.iter_batched(
                || wasm_bytes.clone(),
                |wasm_bytes| {
                    let wasmtime_engine = wasmtime::Engine::default();
                    let wasmtime_module =
                        wasmtime::Module::new(&wasmtime_engine, wasm_bytes).unwrap();
                    let mut wasmtime_store = wasmtime::Store::new(&wasmtime_engine, 5);
                    let wasmtime_instance =
                        wasmtime::Instance::new(&mut wasmtime_store, &wasmtime_module, &[])
                            .unwrap();

                    let res = wasmtime::Val::I32(0);
                    wasmtime_instance
                        .get_func(&mut wasmtime_store, "_start")
                        .unwrap()
                        .call(
                            &mut wasmtime_store,
                            &[wasmtime::Val::I32(*num as i32)],
                            &mut [res],
                        )
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
    targets = wasmtime_fibonacci
);
criterion_main!(benches);
