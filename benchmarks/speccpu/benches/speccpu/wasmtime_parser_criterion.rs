use criterion::{criterion_group, criterion_main, Criterion};
mod util;
use util::*;

fn wasmtime_parser_criterion_505(c: &mut Criterion) {
    util::wasmtime_parser_criterion(SPECCPU_505.clone(), c);
}

fn wasmtime_parser_criterion_508(c: &mut Criterion) {
    util::wasmtime_parser_criterion(SPECCPU_508.clone(), c);
}

fn wasmtime_parser_criterion_519(c: &mut Criterion) {
    util::wasmtime_parser_criterion(SPECCPU_519.clone(), c);
}

fn wasmtime_parser_criterion_557(c: &mut Criterion) {
    util::wasmtime_parser_criterion(SPECCPU_557.clone(), c);
}

criterion_group!(
    name = benches;
    config = Criterion::default();
    targets =
        wasmtime_parser_criterion_505,
        wasmtime_parser_criterion_508,
        wasmtime_parser_criterion_519,
        wasmtime_parser_criterion_557
);
criterion_main!(benches);
