use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput};
use std::path::PathBuf;

const EXACT_ITER_UNTIL: u32 = 30;

pub fn wasmedge_fibonacci(c: &mut Criterion) {
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
            continue;
        }
        group.bench_with_input(BenchmarkId::new("wasmedge", num), &num, |b, num| {
            b.iter_batched(
                || wasm_bytes.clone(),
                |wasm_bytes| {
                    let vm = wasmedge_sdk::VmBuilder::new().build().unwrap();
                    let vm = vm.register_module_from_bytes("main", wasm_bytes).unwrap();

                    vm.run_func(
                        Some("main"),
                        "_start",
                        [wasmedge_sdk::WasmValue::from_i32(*num as i32)],
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
    targets = wasmedge_fibonacci
);
criterion_main!(benches);
