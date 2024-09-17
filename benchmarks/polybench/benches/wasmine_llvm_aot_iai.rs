use std::{path::PathBuf, rc::Rc};

use iai_callgrind::{library_benchmark, library_benchmark_group, main, LibraryBenchmarkConfig};
mod utils;
use module::Module;
use runtime_lib::{ClusterConfig, ResourceBuffer};
use utils::*;
use wasi::WasiContextBuilder;

fn setup_for_compile(bm: &str) -> Vec<u8> {
    let bm_path = get_bm_path(bm);
    assert!(bm_path.exists(), "{} does not exist", bm_path.display());
    std::fs::read(bm_path).unwrap()
}

fn teardown_for_compile(path: PathBuf) {
    std::fs::remove_file(path).unwrap();
}

#[library_benchmark]
#[benches::with_setup(args = [
    "gemm",
    "2mm",
    "trisolv",
    "adi",
    "atax",
    "floyd-warshall",
    "gramschmidt",
    "bicg",
    "syrk",
    "3mm",
    "fdtd-2d",
    "gemver",
    "jacobi-2d",
    "nussinov",
    "seidel-2d",
    "gesummv",
    "correlation",
    "cholesky",
    "jacobi-1d",
    "mvt",
    "heat-3d",
    "symm",
    "deriche",
    "trmm",
    "ludcmp",
    "syr2k",
    "durbin",
    "lu",
    "covariance",
    "doitgen",
], setup = setup_for_compile, teardown = teardown_for_compile)]
pub fn wasmine_llvm_aot_compile(wasm_bytes: Vec<u8>) -> PathBuf {
    let source = ResourceBuffer::from_wasm_buf(wasm_bytes);
    let mut module = module::Module::new(source);
    module.load_meta(parser::ModuleMetaLoader).unwrap();
    module.load_all_functions(parser::FunctionLoader).unwrap();
    let module = Rc::new(module);

    let context = Rc::new(llvm_gen::Context::create());
    let mut executor = llvm_gen::JITExecutor::new(context.clone()).unwrap();

    let llvm_module =
        llvm_gen::Translator::translate_module(context.clone(), module.clone()).unwrap();
    executor.add_module(llvm_module).unwrap();
    let llvm_module_object_buf = executor.get_module_as_object_buffer(0).unwrap();

    let compiled_file_path = tempfile::NamedTempFile::new()
        .unwrap()
        .into_temp_path()
        .with_extension("cwasm");
    module
        .store(
            parser::ModuleStorer,
            llvm_module_object_buf,
            &compiled_file_path,
        )
        .unwrap();

    compiled_file_path
}

fn setup_for_execute(bm: &str) -> PathBuf {
    let bm_path = get_bm_path(bm);
    assert!(bm_path.exists(), "{} does not exist", bm_path.display());
    let wasm_bytes = std::fs::read(bm_path).unwrap();

    let source = ResourceBuffer::from_wasm_buf(wasm_bytes);
    let mut module = module::Module::new(source);
    module.load_meta(parser::ModuleMetaLoader).unwrap();
    module.load_all_functions(parser::FunctionLoader).unwrap();
    let module = Rc::new(module);

    let context = Rc::new(llvm_gen::Context::create());
    let mut executor = llvm_gen::JITExecutor::new(context.clone()).unwrap();
    let llvm_module =
        llvm_gen::Translator::translate_module(context.clone(), module.clone()).unwrap();
    executor.add_module(llvm_module).unwrap();
    let llvm_module_object_buf = executor.get_module_as_object_buffer(0).unwrap();

    let compiled_file_path = tempfile::NamedTempFile::new()
        .unwrap()
        .into_temp_path()
        .with_extension("cwasm");
    module
        .store(
            parser::ModuleStorer,
            llvm_module_object_buf,
            &compiled_file_path,
        )
        .unwrap();
    compiled_file_path
}

fn teardown_for_execute(path: PathBuf) {
    std::fs::remove_file(path).unwrap();
}

#[library_benchmark]
#[benches::with_setup(args = [
    "gemm",
    "2mm",
    "trisolv",
    "adi",
    "atax",
    "floyd-warshall",
    "gramschmidt",
    "bicg",
    "syrk",
    "3mm",
    "fdtd-2d",
    "gemver",
    "jacobi-2d",
    "nussinov",
    "seidel-2d",
    "gesummv",
    "correlation",
    "cholesky",
    "jacobi-1d",
    "mvt",
    "heat-3d",
    "symm",
    "deriche",
    "trmm",
    "ludcmp",
    "syr2k",
    "durbin",
    "lu",
    "covariance",
    "doitgen",
], setup = setup_for_execute, teardown = teardown_for_execute)]
pub fn wasmine_llvm_aot_execute(compiled_file_path: PathBuf) -> PathBuf {
    let source = ResourceBuffer::from_file(&compiled_file_path).unwrap();
    let mut wasmine_engine = runtime_lib::Engine::llvm().unwrap();
    let wasmine_cluster = runtime_lib::Cluster::new(ClusterConfig::default());
    let mut wasmine_module = Module::new(source);
    wasmine_module.load_meta(parser::ModuleMetaLoader).unwrap();
    wasmine_module
        .load_all_functions(parser::FunctionLoader)
        .unwrap();
    let wasmine_module = Rc::new(wasmine_module);
    wasmine_engine.init(wasmine_module.clone()).unwrap();

    let wasi_ctxt = {
        let mut builder = WasiContextBuilder::new();
        builder.inherit_stdio();
        builder.finish()
    };

    let wasmine_instance = runtime_lib::BoundLinker::new(&wasmine_cluster)
        .instantiate_and_link_with_wasi(wasmine_module.clone(), wasmine_engine, wasi_ctxt)
        .unwrap();

    wasmine_instance
        .get_function_by_idx(wasmine_instance.query_start_function().unwrap())
        .unwrap()
        .call(&[])
        .unwrap();

    compiled_file_path
}

library_benchmark_group!(
    name = bench_polybench_group;
    config = LibraryBenchmarkConfig::default().env_clear(false);
    benchmarks = wasmine_llvm_aot_compile, wasmine_llvm_aot_execute
);

main!(library_benchmark_groups = bench_polybench_group);
