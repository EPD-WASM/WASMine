use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::path::PathBuf;
use wasm_rt::parser;

pub fn criterion_benchmark(c: &mut Criterion) {
    let wasm_files = [
        "fibonacci.wasm",
        "regex.wasm",
        "blake3-scalar.wasm",
        "bz2.wasm",
        "image-classification-benchmark.wasm",
    ];
    let mut group = c.benchmark_group("parser-throughput");
    for file_name in wasm_files {
        let wast_bytes = std::fs::read(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("benches/fixtures")
                .join(file_name),
        )
        .unwrap();
        group.throughput(Throughput::Bytes(wast_bytes.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("WASM_RT: parsing", file_name),
            &wast_bytes,
            |b, wast_bytes| {
                b.iter(|| {
                    black_box(
                        parser::parser::Parser::default().parse(black_box(wast_bytes.as_slice())),
                    )
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("WASMTIME: parsing", file_name),
            &wast_bytes,
            |b, wast_bytes| {
                b.iter(|| black_box(wasmparser::validate(black_box(wast_bytes))).unwrap())
            },
        );
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
