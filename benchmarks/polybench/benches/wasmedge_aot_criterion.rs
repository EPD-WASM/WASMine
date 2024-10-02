use std::collections::HashMap;

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput};

mod utils;
use utils::*;
use wasmedge_sdk::{
    config::{CommonConfigOptions, CompilerConfigOptions, ConfigBuilder},
    wasi::WasiModule,
    Compiler, CompilerOptimizationLevel, CompilerOutputFormat, Module, Store, Vm,
};

pub fn wasmedge_aot_criterion(c: &mut Criterion) {
    let mut group = c.benchmark_group("polybench");
    group.throughput(Throughput::Elements(1));
    group.sample_size(10);
    group.sampling_mode(criterion::SamplingMode::Auto);

    for bm in BENCHMARKS {
        let bm_path = get_bm_path(bm);
        assert!(bm_path.exists(), "{} does not exist", bm_path.display());
    }

    for bm in BENCHMARKS {
        let bm_path = get_bm_path(bm);
        let wasm_bytes = std::fs::read(bm_path).unwrap();

        let compiled_file_path = tempfile::NamedTempFile::new()
            .unwrap()
            .into_temp_path()
            .with_extension("so");
        group.bench_with_input(
            BenchmarkId::new("wasmedge_aot_compile", bm),
            &wasm_bytes,
            |b, wasm_bytes| {
                b.iter_batched(
                    || wasm_bytes.clone(),
                    |wasm_bytes| {
                        let _stdout_dropper = gag::Gag::stdout().unwrap();
                        let _stderr_dropper = gag::Gag::stderr().unwrap();

                        let config = ConfigBuilder::new(CommonConfigOptions::default())
                            .with_compiler_config(
                                CompilerConfigOptions::default()
                                    .optimization_level(CompilerOptimizationLevel::O3)
                                    .out_format(CompilerOutputFormat::Wasm)
                                    .dump_ir(false),
                            )
                            .build()
                            .unwrap();
                        let compiler = Compiler::new(Some(&config)).unwrap();
                        compiler.compile_from_bytes(
                            wasm_bytes,
                            compiled_file_path.file_stem().unwrap().to_str().unwrap(),
                            compiled_file_path.parent().unwrap().to_str().unwrap(),
                        )
                    },
                    BatchSize::SmallInput,
                )
            },
        );

        group.bench_with_input(
            BenchmarkId::new("wasmedge_aot_execute", bm),
            &compiled_file_path,
            |b, compiled_file_path| {
                b.iter(|| {
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
                    let module = Module::from_file(None, compiled_file_path).unwrap();
                    vm.register_module(Some("main"), module).unwrap();
                    vm.run_func(Some("main"), "_start", []).unwrap()
                });
            },
        );
        std::fs::remove_file(compiled_file_path).unwrap();
    }
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = wasmedge_aot_criterion
);
criterion_main!(benches);
