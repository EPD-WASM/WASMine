use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use loader::Loader;
use std::path::PathBuf;

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
        let wasm_bytes = std::fs::read(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("benches/fixtures")
                .join(file_name),
        )
        .unwrap();
        group.throughput(Throughput::Bytes(wasm_bytes.len() as u64));
        group.sample_size(50);
        group.bench_with_input(
            BenchmarkId::new("WASM_RT: parsing", file_name),
            &wasm_bytes,
            |b, wasm_bytes| {
                b.iter(|| {
                    black_box(
                        parser::parser::Parser::default()
                            .parse(black_box(Loader::from_buf(wasm_bytes.clone()))),
                    )
                })
            },
        );
        group.bench_with_input(
            BenchmarkId::new("WASMTIME: parsing", file_name),
            &wasm_bytes,
            |b, wasm_bytes| {
                b.iter(|| wasmparser::Validator::new().validate_all(black_box(wasm_bytes)))
            },
        );
    }
    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
