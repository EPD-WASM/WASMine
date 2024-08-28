use std::rc::Rc;

use iai_callgrind::{library_benchmark, library_benchmark_group, main, LibraryBenchmarkConfig};
mod utils;
use runtime_lib::{wasi::WasiContextBuilder, ClusterConfig};
use tempfile::NamedTempFile;
use utils::*;

fn setup_for_compile(bm: &str) -> Vec<u8> {
    let bm_path = get_bm_path(bm);
    assert!(bm_path.exists(), "{} does not exist", bm_path.display());
    std::fs::read(bm_path).unwrap()
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
], setup = setup_for_compile)]
pub fn wasmine_llvm_aot_compile(wasm_bytes: Vec<u8>) {
    let loader = loader::WasmLoader::from_buf(wasm_bytes);
    let parser = parser::Parser::default();
    let module = Rc::new(parser.parse(loader).unwrap());

    let context = Rc::new(llvm_gen::Context::create());
    let mut executor = llvm_gen::JITExecutor::new(context.clone()).unwrap();
    let mut translator = llvm_gen::Translator::new(context.clone()).unwrap();

    let llvm_module = translator.translate_module(module.clone()).unwrap();
    executor.add_module(llvm_module).unwrap();
    let llvm_module_object_buf = executor.get_module_as_object_buffer(0).unwrap();

    let compiled_file = tempfile::NamedTempFile::new().unwrap();
    loader::CwasmLoader::write(compiled_file.path(), module, llvm_module_object_buf).unwrap()
}

fn setup_for_execute(bm: &str) -> NamedTempFile {
    let bm_path = get_bm_path(bm);
    assert!(bm_path.exists(), "{} does not exist", bm_path.display());
    let wasm_bytes = std::fs::read(bm_path).unwrap();

    let loader = loader::WasmLoader::from_buf(wasm_bytes);
    let parser = parser::Parser::default();
    let module = Rc::new(parser.parse(loader).unwrap());

    let context = Rc::new(llvm_gen::Context::create());
    let mut executor = llvm_gen::JITExecutor::new(context.clone()).unwrap();
    let mut translator = llvm_gen::Translator::new(context.clone()).unwrap();

    let llvm_module = translator.translate_module(module.clone()).unwrap();
    executor.add_module(llvm_module).unwrap();
    let llvm_module_object_buf = executor.get_module_as_object_buffer(0).unwrap();

    let compiled_file = tempfile::NamedTempFile::new().unwrap();
    loader::CwasmLoader::write(compiled_file.path(), module, llvm_module_object_buf).unwrap();
    compiled_file
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
], setup = setup_for_execute)]
pub fn wasmine_llvm_aot_execute(compiled_file: NamedTempFile) {
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
        .instantiate_and_link_with_wasi(wasmine_module.clone(), wasmine_engine, wasi_ctxt)
        .unwrap();

    wasmine_instance
        .get_function_by_idx(wasmine_instance.query_start_function().unwrap())
        .unwrap()
        .call(&[])
        .unwrap();
}

library_benchmark_group!(
    name = bench_polybench_group;
    config = LibraryBenchmarkConfig::default().env_clear(false);
    benchmarks = wasmine_llvm_aot_compile, wasmine_llvm_aot_execute
);

main!(library_benchmark_groups = bench_polybench_group);
