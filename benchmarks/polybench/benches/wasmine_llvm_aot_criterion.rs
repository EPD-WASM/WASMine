use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput};
use llvm_gen::LLVMAdditionalResources;
use parser::Parser;
use runtime_lib::{ClusterConfig, FunctionLoaderInterface, ResourceBuffer};
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

        let compiled_file_path = tempfile::NamedTempFile::new()
            .unwrap()
            .into_temp_path()
            .with_extension("cwasm");
        group.bench_with_input(
            BenchmarkId::new("wasmine_llvm_aot_compile", bm),
            &wasm_bytes,
            |b, wasm_bytes| {
                b.iter_batched(
                    || wasm_bytes.clone(),
                    |wasm_bytes| {
                        let _stdout_dropper = gag::Gag::stdout().unwrap();
                        let _stderr_dropper = gag::Gag::stderr().unwrap();

                        let module = Parser::parse_from_buf(wasm_bytes).unwrap();
                        llvm_gen::Translator::translate_module_meta(&module).unwrap();
                        llvm_gen::FunctionLoader
                            .parse_all_functions(&module)
                            .unwrap();
                        let module = Rc::new(module);

                        let (llvm_module, context) = {
                            let artifacts_ref = module.artifact_registry.read().unwrap();
                            let llvm_resources =
                                artifacts_ref.get("llvm-module").unwrap().read().unwrap();
                            let llvm_resources = llvm_resources
                                .downcast_ref::<LLVMAdditionalResources>()
                                .unwrap();
                            (
                                llvm_resources.module.clone(),
                                llvm_resources.context.clone(),
                            )
                        };
                        let mut executor = llvm_gen::JITExecutor::new(context.clone()).unwrap();
                        executor.add_module(llvm_module).unwrap();
                        let llvm_module_object_buf =
                            executor.get_module_as_object_buffer(0).unwrap();
                        llvm_gen::aot::store_aot_module(
                            &module,
                            llvm_module_object_buf,
                            &compiled_file_path,
                        )
                        .unwrap()
                    },
                    BatchSize::SmallInput,
                )
            },
        );

        group.bench_with_input(
            BenchmarkId::new("wasmine_llvm_aot_execute", bm),
            &compiled_file_path,
            |b, compiled_file_path| {
                b.iter(|| {
                    let _stdout_dropper = gag::Gag::stdout().unwrap();
                    let _stderr_dropper = gag::Gag::stderr().unwrap();

                    let source = ResourceBuffer::from_file(compiled_file_path).unwrap();
                    let mut wasmine_engine = runtime_lib::Engine::llvm().unwrap();
                    let wasmine_cluster = runtime_lib::Cluster::new(ClusterConfig::default());

                    let module = llvm_gen::aot::parse_aot_module(source).unwrap();
                    let module = Rc::new(module);
                    wasmine_engine.init(module.clone()).unwrap();

                    let wasi_ctxt = {
                        let mut builder = WasiContextBuilder::new();
                        builder.inherit_stdio();
                        builder.finish()
                    };

                    let wasmine_instance = runtime_lib::BoundLinker::new(&wasmine_cluster)
                        .instantiate_and_link_with_wasi(module.clone(), wasmine_engine, wasi_ctxt)
                        .unwrap();

                    wasmine_instance
                        .get_function_by_idx(wasmine_instance.query_start_function().unwrap())
                        .unwrap()
                        .call(&[])
                        .unwrap();
                });
            },
        );
        std::fs::remove_file(compiled_file_path).unwrap();
    }
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = wasmine_llvm_aot_criterion
);
criterion_main!(benches);
