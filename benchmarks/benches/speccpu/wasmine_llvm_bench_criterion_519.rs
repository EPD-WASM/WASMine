use criterion::{criterion_group, criterion_main, Criterion};
mod util;
use util::*;

fn wasmine_llvm_speccpu_criterion_519(c: &mut Criterion) {
    util::wasmine_llvm_speccpu_criterion(SPECCPU_519.clone(), c);
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = wasmine_llvm_speccpu_criterion_519
);
criterion_main!(benches);
