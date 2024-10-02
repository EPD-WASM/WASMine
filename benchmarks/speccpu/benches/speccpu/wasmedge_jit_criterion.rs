use criterion::{criterion_group, criterion_main, Criterion};
mod util;
use util::*;

fn wasmedge_jit_criterion_505(c: &mut Criterion) {
    util::wasmedge_jit_criterion(SPECCPU_505.clone(), c);
}

fn wasmedge_jit_criterion_508(c: &mut Criterion) {
    util::wasmedge_jit_criterion(SPECCPU_508.clone(), c);
}

fn wasmedge_jit_criterion_519(c: &mut Criterion) {
    util::wasmedge_jit_criterion(SPECCPU_519.clone(), c);
}

fn wasmedge_jit_criterion_557(c: &mut Criterion) {
    util::wasmedge_jit_criterion(SPECCPU_557.clone(), c);
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets =
        wasmedge_jit_criterion_505,
        wasmedge_jit_criterion_508,
        wasmedge_jit_criterion_519,
        wasmedge_jit_criterion_557
);
criterion_main!(benches);
