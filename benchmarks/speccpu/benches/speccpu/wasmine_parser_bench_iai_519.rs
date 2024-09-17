use iai_callgrind::{library_benchmark, library_benchmark_group, main, LibraryBenchmarkConfig};
use util::*;
mod util;

#[library_benchmark]
pub fn wasmine_parse_iai_519() -> runtime_lib::WasmModule {
    util::wasmine_parse_speccpu_iai(SPECCPU_519.clone())
}

library_benchmark_group!(
    name = bench_speccpu_group;
    config = LibraryBenchmarkConfig::default().env_clear(false);
    benchmarks = wasmine_parse_iai_519
);

main!(library_benchmark_groups = bench_speccpu_group);
