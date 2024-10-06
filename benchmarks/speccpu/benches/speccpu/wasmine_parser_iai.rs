use iai_callgrind::{library_benchmark, library_benchmark_group, main, LibraryBenchmarkConfig};
use std::rc::Rc;
use util::{SPECCPU_505, SPECCPU_508, SPECCPU_519, SPECCPU_557};
mod util;

#[library_benchmark]
pub fn wasmine_parse_iai_505() -> Rc<runtime_lib::WasmModule> {
    util::wasmine_parse_iai(SPECCPU_505.clone())
}

#[library_benchmark]
pub fn wasmine_parse_iai_508() -> Rc<runtime_lib::WasmModule> {
    util::wasmine_parse_iai(SPECCPU_508.clone())
}

#[library_benchmark]
pub fn wasmine_parse_iai_519() -> Rc<runtime_lib::WasmModule> {
    util::wasmine_parse_iai(SPECCPU_519.clone())
}

#[library_benchmark]
pub fn wasmine_parse_iai_557() -> Rc<runtime_lib::WasmModule> {
    util::wasmine_parse_iai(SPECCPU_557.clone())
}

library_benchmark_group!(
    name = bench_group;
    config = LibraryBenchmarkConfig::default().env_clear(false);
    benchmarks =
        wasmine_parse_iai_505,
        wasmine_parse_iai_508,
        wasmine_parse_iai_519,
        wasmine_parse_iai_557
);

main!(library_benchmark_groups = bench_group);
