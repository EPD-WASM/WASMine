use iai_callgrind::{library_benchmark, library_benchmark_group, main, LibraryBenchmarkConfig};
use util::*;
mod util;

#[library_benchmark]
pub fn wasmine_llvm_e2e_iai_519() {
    util::wasmine_llvm_speccpu_iai(SPECCPU_519.clone());
}

library_benchmark_group!(
    name = bench_speccpu_group;
    config = LibraryBenchmarkConfig::default().raw_callgrind_args(["--cache-sim=no"]).env_clear(false);
    benchmarks = wasmine_llvm_e2e_iai_519
);

main!(library_benchmark_groups = bench_speccpu_group);
