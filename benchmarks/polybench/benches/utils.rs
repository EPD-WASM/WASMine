use std::{env, path::PathBuf};

#[allow(dead_code)]
pub const BENCHMARKS: &[&str] = &[
    "gemm",
    "2mm",
    "trisolv",
    "adi",
    "atax",
    "floyd-warshall",
    "gramschmidt",
    "bicg",
    "syrk",
    "3mm",
    "fdtd-2d",
    "gemver",
    "jacobi-2d",
    "nussinov",
    "seidel-2d",
    "gesummv",
    "correlation",
    "cholesky",
    "jacobi-1d",
    "mvt",
    "heat-3d",
    "symm",
    "deriche",
    "trmm",
    "ludcmp",
    "syr2k",
    "durbin",
    "lu",
    "covariance",
    "doitgen",
];

pub fn get_bm_path(bm: &str) -> PathBuf {
    PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR environment variable not set"))
        .join("polybench")
        .join(bm)
        .with_extension("wasm")
}
