use criterion::{criterion_group, criterion_main, Criterion};
mod util;
use util::*;

fn wasmine_parser_speccpu_criterion_505(c: &mut Criterion) {
    util::wasmine_parser_speccpu_criterion(SPECCPU_505.clone(), c);
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets = wasmine_parser_speccpu_criterion_505
);
criterion_main!(benches);
