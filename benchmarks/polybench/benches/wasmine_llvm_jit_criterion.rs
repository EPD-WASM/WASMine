use std::rc::Rc;

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion, Throughput};
use parser::Parser;
use runtime_lib::{ClusterConfig, FunctionLoaderInterface};
use wasi::WasiContextBuilder;

mod utils;
use utils::*;

pub fn wasmine_llvm_jit_criterion(c: &mut Criterion) {
    let mut group = c.benchmark_group("polybench");
    group.throughput(Throughput::Elements(1));
    group.sample_size(10);
    group.warm_up_time(std::time::Duration::from_secs(5));
    group.sampling_mode(criterion::SamplingMode::Flat);

    for bm in BENCHMARKS {
        let bm_path = get_bm_path(bm);
        let wasm_bytes = std::fs::read(bm_path).unwrap();
        group.bench_with_input(
            BenchmarkId::new("wasmine_llvm_jit", bm),
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

                        let wasmine_cluster = runtime_lib::Cluster::new(ClusterConfig::default());
                        let mut wasmine_engine = runtime_lib::Engine::llvm().unwrap();
                        wasmine_engine.init(module.clone()).unwrap();

                        let wasi_ctxt = {
                            let mut builder = WasiContextBuilder::new();
                            builder.inherit_stdio();
                            builder.finish()
                        };

                        let wasmine_instance = runtime_lib::BoundLinker::new(&wasmine_cluster)
                            .instantiate_and_link_with_wasi(
                                module.clone(),
                                wasmine_engine,
                                wasi_ctxt,
                            )
                            .unwrap();

                        wasmine_instance
                            .get_function_by_idx(wasmine_instance.query_start_function().unwrap())
                            .unwrap()
                            .call(&[])
                            .unwrap();
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
    targets = wasmine_llvm_jit_criterion
);
criterion_main!(benches);
