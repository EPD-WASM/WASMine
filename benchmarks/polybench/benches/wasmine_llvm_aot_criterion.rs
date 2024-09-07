use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput};
use runtime_lib::ClusterConfig;
use std::rc::Rc;
use wasi::WasiContextBuilder;

mod utils;
use utils::*;

pub fn wasmine_llvm_aot_criterion(c: &mut Criterion) {
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

        let compiled_file = tempfile::NamedTempFile::new().unwrap();
        group.bench_with_input(
            BenchmarkId::new("wasmine_llvm_aot_compile", bm),
            &wasm_bytes,
            |b, wasm_bytes| {
                b.iter_batched(
                    || wasm_bytes.clone(),
                    |wasm_bytes| {
                        let _stdout_dropper = gag::Gag::stdout().unwrap();
                        let _stderr_dropper = gag::Gag::stderr().unwrap();

                        let loader = loader::WasmLoader::from_buf(wasm_bytes);
                        let parser = parser::Parser::default();
                        let module = Rc::new(parser.parse(loader).unwrap());

                        let context = Rc::new(llvm_gen::Context::create());
                        let mut executor = llvm_gen::JITExecutor::new(context.clone()).unwrap();
                        let mut translator = llvm_gen::Translator::new(context.clone()).unwrap();

                        let llvm_module = translator.translate_module(module.clone()).unwrap();
                        executor.add_module(llvm_module).unwrap();
                        let llvm_module_object_buf =
                            executor.get_module_as_object_buffer(0).unwrap();
                        loader::CwasmLoader::write(
                            compiled_file.path(),
                            module,
                            llvm_module_object_buf,
                        )
                        .unwrap()
                    },
                    BatchSize::SmallInput,
                )
            },
        );

        group.bench_with_input(
            BenchmarkId::new("wasmine_llvm_aot_execute", bm),
            &compiled_file,
            |b, compiled_file| {
                b.iter(|| {
                    let _stdout_dropper = gag::Gag::stdout().unwrap();
                    let _stderr_dropper = gag::Gag::stderr().unwrap();

                    let loader = loader::CwasmLoader::from_file(compiled_file.path()).unwrap();
                    let mut wasmine_engine = runtime_lib::Engine::llvm().unwrap();
                    let wasmine_cluster = runtime_lib::Cluster::new(ClusterConfig::default());
                    let wasmine_module = loader.wasm_module();
                    wasmine_engine
                        .init(wasmine_module.clone(), Some(&loader))
                        .unwrap();

                    let wasi_ctxt = {
                        let mut builder = WasiContextBuilder::new();
                        builder.inherit_stdio();
                        builder.finish()
                    };

                    let wasmine_instance = runtime_lib::BoundLinker::new(&wasmine_cluster)
                        .instantiate_and_link_with_wasi(
                            wasmine_module.clone(),
                            wasmine_engine,
                            wasi_ctxt,
                        )
                        .unwrap();

                    wasmine_instance
                        .get_function_by_idx(wasmine_instance.query_start_function().unwrap())
                        .unwrap()
                        .call(&[])
                        .unwrap();
                });
            },
        );
    }
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = wasmine_llvm_aot_criterion
);
criterion_main!(benches);
