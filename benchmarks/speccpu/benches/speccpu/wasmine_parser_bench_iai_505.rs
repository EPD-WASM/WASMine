use iai_callgrind::{library_benchmark, library_benchmark_group, main, LibraryBenchmarkConfig};
use util::SPECCPU_505;
mod util;

#[library_benchmark]
pub fn wasmine_parse_iai_505() -> runtime_lib::WasmModule {
    util::wasmine_parse_speccpu_iai(SPECCPU_505.clone())
}

library_benchmark_group!(
    name = bench_speccpu_group;
    config = LibraryBenchmarkConfig::default().env_clear(false);
    benchmarks = wasmine_parse_iai_505
);

main!(library_benchmark_groups = bench_speccpu_group);
