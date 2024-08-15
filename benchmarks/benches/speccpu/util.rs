#![allow(dead_code)]
use criterion::{BatchSize, BenchmarkId, Criterion, Throughput};
use loader::Loader;
use std::{path::PathBuf, rc::Rc, sync::LazyLock};

pub static PATH_505: LazyLock<PathBuf> =
    LazyLock::new(|| PathBuf::from(std::env::var_os("PATH_505").unwrap()));
pub static PATH_508: LazyLock<PathBuf> =
    LazyLock::new(|| PathBuf::from(std::env::var_os("PATH_508").unwrap()));
pub static PATH_519: LazyLock<PathBuf> =
    LazyLock::new(|| PathBuf::from(std::env::var_os("PATH_519").unwrap()));
pub static PATH_557: LazyLock<PathBuf> =
    LazyLock::new(|| PathBuf::from(std::env::var_os("PATH_557").unwrap()));

#[derive(Debug, Clone)]
pub struct SpeccpuBenchmark {
    pub name: String,
    pub wasm_path: PathBuf,
    pub dirs: Vec<(PathBuf, String)>,
    pub args: Vec<String>,
}

pub static SPECCPU_505: LazyLock<SpeccpuBenchmark> = LazyLock::new(|| SpeccpuBenchmark {
    name: "505.mcf_r".into(),
    wasm_path: PATH_505.join("unpatched.wasm"),
    dirs: vec![(PATH_505.clone(), ".".to_string())],
    args: vec![
        "505.mcf_r".to_string(),
        "./data/test/input/inp.in".to_string(),
    ],
});

pub static SPECCPU_508: LazyLock<SpeccpuBenchmark> = LazyLock::new(|| SpeccpuBenchmark {
    name: "508.namd_r".into(),
    wasm_path: PATH_508.join("unpatched.wasm"),
    dirs: vec![(PATH_508.clone(), ".".to_string())],
    args: vec![
        "508.namd_r".to_string(),
        "--input".to_string(),
        "./data/all/input/apoa1.input".to_string(),
        "--iterations".to_string(),
        "1".to_string(),
        "--output".to_string(),
        "apoa1.test.output".to_string(),
    ],
});

pub static SPECCPU_519: LazyLock<SpeccpuBenchmark> = LazyLock::new(|| SpeccpuBenchmark {
    name: "519.lbm_r".into(),
    wasm_path: PATH_519.join("unpatched.wasm"),
    dirs: vec![(PATH_519.clone(), ".".to_string())],
    args: vec![
        "519.lbm_r".to_string(),
        "20".to_string(),
        "reference.dat".to_string(),
        "0".to_string(),
        "1".to_string(),
        "./data/test/input/100_100_130_cf_a.of".to_string(),
    ],
});

pub static SPECCPU_557: LazyLock<SpeccpuBenchmark> = LazyLock::new(|| {
    SpeccpuBenchmark {
    name: "557.xz_r".into(),
    wasm_path: PATH_557.join("unpatched.wasm"),
    dirs: vec![(PATH_557.clone(), ".".to_string())],
    args: vec![
        "557.xz_r".to_string(),
        "./data/all/input/cpu2006docs.tar.xz".to_string(),
        "4".to_string(),
        "055ce243071129412e9dd0b3b69a21654033a9b723d874b2015c774fac1553d9713be561ca86f74e4f16f22e664fc17a79f30caa5ad2c04fbc447549c2810fae".to_string(),
        "1548636".to_string(),
        "1555348".to_string(),
        "0".to_string(),
    ],
}
});

pub fn wasmine_llvm_speccpu_criterion(bm: SpeccpuBenchmark, c: &mut Criterion) {
    let mut group = c.benchmark_group("speccpu");
    group.throughput(Throughput::Elements(1));
    group.sample_size(10);
    group.warm_up_time(std::time::Duration::from_secs(5));
    group.sampling_mode(criterion::SamplingMode::Flat);

    let wasm_bytes = std::fs::read(bm.wasm_path).unwrap();
    group.bench_function(BenchmarkId::new("wasmine_llvm_speccpu", bm.name), |b| {
        b.iter_batched(
            || wasm_bytes.clone(),
            |wasm_bytes| {
                let _stdout_dropper = gag::Gag::stdout().unwrap();
                let _stderr_dropper = gag::Gag::stderr().unwrap();

                let wasmine_module = Rc::new(
                    parser::parser::Parser::default()
                        .parse(Loader::from_buf(wasm_bytes))
                        .unwrap(),
                );
                let config = runtime_lib::ConfigBuilder::new()
                    .enable_wasi(true)
                    .set_wasi_dirs(bm.dirs.clone())
                    .set_wasi_args(bm.args.clone())
                    .finish();
                let wasmine_cluster = runtime_lib::Cluster::new(config);
                let mut wasmine_engine = runtime_lib::Engine::llvm().unwrap();
                wasmine_engine.init(wasmine_module.clone()).unwrap();

                let wasmine_instance = runtime_lib::BoundLinker::new(&wasmine_cluster)
                    .instantiate_and_link(wasmine_module.clone(), wasmine_engine)
                    .unwrap();

                wasmine_instance
                    .get_function_by_idx(wasmine_instance.query_start_function().unwrap())
                    .unwrap()
                    .call(&[])
                    .unwrap();
            },
            BatchSize::SmallInput,
        );
    });
}

pub fn wasmine_llvm_speccpu_iai(bm: SpeccpuBenchmark) {
    let _stdout_dropper = gag::Gag::stdout().unwrap();
    let _stderr_dropper = gag::Gag::stderr().unwrap();

    let wasm_file_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(bm.wasm_path);

    let wasm_bytes = std::fs::read(wasm_file_path).unwrap();
    let wasmine_module = Rc::new(
        parser::parser::Parser::default()
            .parse(Loader::from_buf(wasm_bytes.clone()))
            .unwrap(),
    );
    let config = runtime_lib::ConfigBuilder::new()
        .enable_wasi(true)
        .set_wasi_dirs(vec![])
        .set_wasi_args(vec![])
        .finish();
    let wasmine_cluster = runtime_lib::Cluster::new(config);
    let mut wasmine_engine = runtime_lib::Engine::llvm().unwrap();
    wasmine_engine.init(wasmine_module.clone()).unwrap();

    let wasmine_instance = runtime_lib::BoundLinker::new(&wasmine_cluster)
        .instantiate_and_link(wasmine_module.clone(), wasmine_engine)
        .unwrap();

    wasmine_instance
        .get_function_by_idx(wasmine_instance.query_start_function().unwrap())
        .unwrap()
        .call(&[])
        .unwrap();
}

pub fn wasmtime_speccpu_criterion(bm: SpeccpuBenchmark, c: &mut Criterion) {
    let mut group = c.benchmark_group("speccpu");
    group.throughput(Throughput::Elements(1));
    group.sample_size(10);
    group.warm_up_time(std::time::Duration::from_secs(5));
    group.sampling_mode(criterion::SamplingMode::Flat);

    let wasm_bytes = std::fs::read(bm.wasm_path).unwrap();
    group.bench_function(BenchmarkId::new("wasmtime_speccpu", bm.name), |b| {
        b.iter_batched(
            || wasm_bytes.clone(),
            |wasm_bytes| {
                let _stdout_dropper = gag::Gag::stdout().unwrap();
                let _stderr_dropper = gag::Gag::stderr().unwrap();

                let engine = wasmtime::Engine::default();
                let mut linker =
                    wasmtime::Linker::<wasmtime_wasi::preview1::WasiP1Ctx>::new(&engine);
                wasmtime_wasi::preview1::add_to_linker_sync(&mut linker, |t| t).unwrap();
                let mut wasi_ctx_builder = wasmtime_wasi::WasiCtxBuilder::new();
                wasi_ctx_builder.inherit_stdio().args(&bm.args);
                for dir in bm.dirs.iter() {
                    wasi_ctx_builder
                        .preopened_dir(
                            dir.0.clone(),
                            dir.1.clone(),
                            wasmtime_wasi::DirPerms::all(),
                            wasmtime_wasi::FilePerms::all(),
                        )
                        .unwrap();
                }
                let wasi_ctx: wasmtime_wasi::preview1::WasiP1Ctx = wasi_ctx_builder.build_p1();
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
    });
}
