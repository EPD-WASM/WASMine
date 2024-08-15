use criterion::{criterion_group, criterion_main, Criterion};
mod util;
use util::*;

fn wasmine_llvm_speccpu_criterion_508(c: &mut Criterion) {
    util::wasmine_llvm_speccpu_criterion(SPECCPU_508.clone(), c);
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = wasmine_llvm_speccpu_criterion_508
);
criterion_main!(benches);
