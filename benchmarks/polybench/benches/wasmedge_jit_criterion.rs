use std::collections::HashMap;

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput};

mod utils;
use utils::*;
use wasmedge_sdk::{
    config::{CommonConfigOptions, ConfigBuilder},
    wasi::WasiModule,
    Module, Store, Vm,
};

// the benchmarks that actually terminate (< 1min per run) with wasmedge jit
pub const WASMEDGE_BENCHMARKS: &[&str] = &[
    "trisolv",
    "atax",
    "bicg",
    "gemver",
    "gesummv",
    "jacobi-1d",
    "mvt",
    "deriche",
    "durbin",
];

pub fn wasmedge_jit_criterion(c: &mut Criterion) {
    let mut group = c.benchmark_group("polybench");
    group.throughput(Throughput::Elements(1));
    group.sample_size(10);
    group.warm_up_time(std::time::Duration::from_secs(5));
    group.sampling_mode(criterion::SamplingMode::Flat);

    for bm in WASMEDGE_BENCHMARKS {
        let bm_path = get_bm_path(bm);
        let wasm_bytes = std::fs::read(bm_path).unwrap();
        group.bench_function(BenchmarkId::new("wasmedge_jit", bm), |b| {
            b.iter_batched(
                || wasm_bytes.clone(),
                |wasm_bytes| {
                    let _stdout_dropper = gag::Gag::stdout().unwrap();
                    let _stderr_dropper = gag::Gag::stderr().unwrap();

                    let mut wasi_module = WasiModule::create(None, None, None).unwrap();
                    let mut instances = HashMap::new();
                    instances.insert(wasi_module.name().to_string(), wasi_module.as_mut());

                    let config =
                        ConfigBuilder::new(CommonConfigOptions::default().interpreter_mode(false))
                            .build()
                            .unwrap();
                    let store = Store::new(Some(&config), instances).unwrap();
                    let mut vm = Vm::new(store);
                    let module = Module::from_bytes(None, wasm_bytes).unwrap();
                    vm.register_module(Some("main"), module).unwrap();
                    vm.run_func(Some("main"), "_start", []).unwrap()
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
    targets = wasmedge_jit_criterion
);
criterion_main!(benches);
