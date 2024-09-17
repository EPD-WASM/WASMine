use std::{env, path::PathBuf, process::Command};

const POLYBENCH_URL: &str =
    "https://sourceforge.net/projects/polybench/files/polybench-c-4.2.1-beta.tar.gz/download";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let polybench_dir = PathBuf::from(env::var("OUT_DIR").unwrap()).join("polybench");
    if !polybench_dir.exists() {
        println!("cargo:warning=missing polybench, starting download");
        let polybench_tar = polybench_dir.with_extension("tar.gz");
        Command::new("curl")
            .arg("-L")
            .arg("-o")
            .arg(polybench_tar.clone())
            .arg(POLYBENCH_URL)
            .status()
            .unwrap();
        assert!(polybench_tar.exists());
        std::fs::create_dir(&polybench_dir).unwrap();
        Command::new("tar")
            .arg("-xzf")
            .arg(polybench_tar)
            .arg("-C")
            .arg(polybench_dir.clone())
            .arg("--strip-components=1")
            .status()
            .unwrap();
        assert!(polybench_dir.exists());
        assert!(polybench_dir.is_dir());
    }

    let available_benchmarks =
        std::fs::read_to_string(polybench_dir.join("utilities/benchmark_list")).unwrap();
    let available_benchmarks = available_benchmarks.split("\n").collect::<Vec<_>>();

    for bm in available_benchmarks {
        if bm.is_empty() {
            continue;
        }
        let bm_path = PathBuf::from(bm);
        let bm_wasm_file = polybench_dir
            .join(bm_path.file_name().unwrap().to_str().unwrap())
            .with_extension("wasm");
        if !bm_wasm_file.exists() {
            println!(
                "cargo:warning=missing benchmark file for {} ({:?}), compiling new using wasi-sdk",
                bm, bm_wasm_file
            );
            let clang_exec =
                PathBuf::from(env::var("WASI_SDK").expect(
                    "WASI_SDK environment variable pointing to valid wasi-sdk installation",
                ))
                .join("bin/clang");
            Command::new(clang_exec)
                .arg("-O3")
                .args(["-I", polybench_dir.join("utilities").to_str().unwrap()])
                .args([
                    "-I",
                    polybench_dir.join(bm).parent().unwrap().to_str().unwrap(),
                ])
                .arg(polybench_dir.join("utilities/polybench.c"))
                .arg(polybench_dir.join(bm))
                .arg("-o")
                .arg(bm_wasm_file)
                .arg("-D_WASI_EMULATED_PROCESS_CLOCKS")
                .arg("-lwasi-emulated-process-clocks")
                .status()
                .unwrap();
        }
    }
}
