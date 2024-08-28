use std::io::Write;

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput};

mod utils;
use utils::*;

pub fn wasmtime_aot_polybench_criterion(c: &mut Criterion) {
    let mut group = c.benchmark_group("polybench");
    group.throughput(Throughput::Elements(1));
    group.sample_size(10);
    group.sampling_mode(criterion::SamplingMode::Auto);

    for bm in BENCHMARKS {
        let bm_path = get_bm_path(bm);
        let wasm_bytes = std::fs::read(bm_path).unwrap();

        let mut compiled_file = tempfile::NamedTempFile::new().unwrap();
        group.bench_with_input(
            BenchmarkId::new("wasmtime_aot_compile", bm),
            &wasm_bytes,
            |b, wasm_bytes| {
                b.iter_batched(
                    || wasm_bytes.clone(),
                    |wasm_bytes| {
                        let _stdout_dropper = gag::Gag::stdout().unwrap();
                        let _stderr_dropper = gag::Gag::stderr().unwrap();

                        let mut config = wasmtime::Config::new();
                        config.strategy(wasmtime::Strategy::Cranelift);
                        config.cranelift_opt_level(wasmtime::OptLevel::Speed);
                        config.disable_cache();

                        let engine = wasmtime::Engine::new(&config).unwrap();
                        let compiled_module =
                            engine.precompile_module(wasm_bytes.as_slice()).unwrap();

                        compiled_file.write_all(compiled_module.as_slice()).unwrap();
                    },
                    BatchSize::SmallInput,
                )
            },
        );

        group.bench_with_input(
            BenchmarkId::new("wasmtime_aot_execute", bm),
            &compiled_file,
            |b, compiled_file| {
                b.iter(|| {
                    let _stdout_dropper = gag::Gag::stdout().unwrap();
                    let _stderr_dropper = gag::Gag::stderr().unwrap();

                    let mut config = wasmtime::Config::new();
                    config.strategy(wasmtime::Strategy::Cranelift);

                    let engine = wasmtime::Engine::new(&config).unwrap();
                    let mut linker =
                        wasmtime::Linker::<wasmtime_wasi::preview1::WasiP1Ctx>::new(&engine);
                    wasmtime_wasi::preview1::add_to_linker_sync(&mut linker, |t| t).unwrap();
                    let mut wasi_ctx_builder = wasmtime_wasi::WasiCtxBuilder::new();
                    wasi_ctx_builder.inherit_stdio();
                    let wasi_ctx: wasmtime_wasi::preview1::WasiP1Ctx = wasi_ctx_builder.build_p1();
                    let mut store = wasmtime::Store::new(&engine, wasi_ctx);

                    let module = unsafe {
                        wasmtime::Module::deserialize_file(&engine, compiled_file.path()).unwrap()
                    };
                    linker
                        .instantiate(&mut store, &module)
                        .unwrap()
                        .get_func(&mut store, "_start")
                        .unwrap()
                        .call(&mut store, &[], &mut [])
                        .unwrap()
                });
            },
        );
    }
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = wasmtime_aot_polybench_criterion
);
criterion_main!(benches);
