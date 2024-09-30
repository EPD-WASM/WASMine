use iai_callgrind::{
    library_benchmark, library_benchmark_group, main, FlamegraphConfig, LibraryBenchmarkConfig,
};
use util::*;
mod util;

#[library_benchmark]
pub fn wasmine_llvm_speccpu_519() {
    util::wasmine_llvm_jit_iai(SPECCPU_519.clone())
}

library_benchmark_group!(
    name = bench_speccpu_group;
    config = LibraryBenchmarkConfig::default().env_clear(false).flamegraph(FlamegraphConfig::default());
    benchmarks = wasmine_llvm_speccpu_519
);

main!(library_benchmark_groups = bench_speccpu_group);
