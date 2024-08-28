use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput};

mod utils;
use utils::*;

pub fn wasmtime_jit_polybench_criterion(c: &mut Criterion) {
    let mut group = c.benchmark_group("polybench");
    group.throughput(Throughput::Elements(1));
    group.sample_size(10);
    group.warm_up_time(std::time::Duration::from_secs(5));
    group.sampling_mode(criterion::SamplingMode::Flat);

    for bm in BENCHMARKS {
        let bm_path = get_bm_path(bm);
        let wasm_bytes = std::fs::read(bm_path).unwrap();
        group.bench_with_input(
            BenchmarkId::new("wasmtime_jit", bm),
            &wasm_bytes,
            |b, wasm_bytes| {
                b.iter_batched(
                    || wasm_bytes.clone(),
                    |wasm_bytes| {
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
                        let wasi_ctx: wasmtime_wasi::preview1::WasiP1Ctx =
                            wasi_ctx_builder.build_p1();
                        let mut store = wasmtime::Store::new(&engine, wasi_ctx);
                        let module = wasmtime::Module::new(&engine, wasm_bytes).unwrap();
                        linker
                            .instantiate(&mut store, &module)
                            .unwrap()
                            .get_func(&mut store, "_start")
                            .unwrap()
                            .call(&mut store, &[], &mut [])
                            .unwrap()
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = wasmtime_jit_polybench_criterion
);
criterion_main!(benches);
