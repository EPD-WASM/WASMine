use std::{path::PathBuf, rc::Rc};

use iai_callgrind::{library_benchmark, library_benchmark_group, main, LibraryBenchmarkConfig};
mod utils;
use parser::Parser;
use runtime_lib::{ClusterConfig, FunctionLoaderInterface, ResourceBuffer};
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
    let module = Parser::parse_from_buf(wasm_bytes).unwrap();
    llvm_gen::Translator::translate_module_meta(&module).unwrap();
    llvm_gen::FunctionLoader
        .parse_all_functions(&module)
        .unwrap();
    let module = Rc::new(module);
    let compiled_file_path = tempfile::NamedTempFile::new()
        .unwrap()
        .into_temp_path()
        .with_extension("cwasm");
    llvm_gen::aot::store_aot_module(module, &compiled_file_path).unwrap();

    compiled_file_path
}

fn setup_for_execute(bm: &str) -> PathBuf {
    let bm_path = get_bm_path(bm);
    assert!(bm_path.exists(), "{} does not exist", bm_path.display());
    let wasm_bytes = std::fs::read(bm_path).unwrap();

    let module = Parser::parse_from_buf(wasm_bytes).unwrap();
    llvm_gen::Translator::translate_module_meta(&module).unwrap();
    llvm_gen::FunctionLoader
        .parse_all_functions(&module)
        .unwrap();
    let module = Rc::new(module);
    let compiled_file_path = tempfile::NamedTempFile::new()
        .unwrap()
        .into_temp_path()
        .with_extension("cwasm");
    llvm_gen::aot::store_aot_module(module, &compiled_file_path).unwrap();
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
    let module = llvm_gen::aot::parse_aot_module(source).unwrap();
    let module = Rc::new(module);
    wasmine_engine.init(module.clone()).unwrap();

    let wasi_ctxt = {
        let mut builder = WasiContextBuilder::new();
        builder.inherit_stdio();
        builder.finish()
    };

    let wasmine_instance = runtime_lib::BoundLinker::new(&wasmine_cluster)
        .instantiate_and_link_with_wasi(module, wasmine_engine, wasi_ctxt)
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
