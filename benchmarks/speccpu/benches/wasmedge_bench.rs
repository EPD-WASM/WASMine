use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput};
use std::{collections::HashMap, path::PathBuf};
use wasmedge_sdk::{wasi::WasiModule, Module, Store, Vm};

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
                    let mut wasi_module =
                        WasiModule::create(None, None, Some(vec![".:."])).unwrap();
                    let mut instances = HashMap::new();
                    instances.insert(wasi_module.name().to_string(), wasi_module.as_mut());
                    let store = Store::new(None, instances).unwrap();
                    let mut vm = Vm::new(store);
                    let module = Module::from_bytes(None, wasm_bytes).unwrap();
                    vm.register_module(Some("wasm-lib"), module).unwrap();

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
