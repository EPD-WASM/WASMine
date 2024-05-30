use std::{collections::HashSet, path::PathBuf};

pub struct SpecTests {}

impl SpecTests {
    pub fn get_files() -> Vec<PathBuf> {
        let project_root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
        let spec_test_files = project_root.join("thirdparty/spec/test/core/");
        if !spec_test_files.exists() {
            panic!("Spec test files not found at {:?}", spec_test_files);
        }
        let mut files = Vec::new();
        for entry in std::fs::read_dir(spec_test_files).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() && path.extension().is_some() && path.extension().unwrap() == "wast" {
                files.push(path);
            }
        }
        files
    }

    fn compile_all_raw_files() {
        let project_root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
        let spec_test_files = project_root.join("thirdparty/spec/test/core/");
        if !spec_test_files.exists() {
            panic!("Spec test files not found at {:?}", spec_test_files);
        }
        let mut raw_files = Vec::new();
        let mut compiled_files = HashSet::new();
        for entry in std::fs::read_dir(spec_test_files).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_file() {
                if let Some(extension) = path.extension() {
                    if path
                        .components()
                        .last()
                        .unwrap()
                        .as_os_str()
                        .to_str()
                        .unwrap()
                        .ends_with(".bin.wast")
                    {
                        compiled_files.insert(path.clone());
                    } else if extension == "wast" {
                        raw_files.push(path);
                    }
                }
            }
        }
        for raw_file in raw_files {
            if !compiled_files.contains(&raw_file.with_extension("bin.wast")) {
                compiled_files.insert(SpecTests::compile(&raw_file));
            }
        }
    }

    fn compile(path: &PathBuf) -> PathBuf {
        let spec_interp_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
            .join("thirdparty/spec/interpreter/");
        let spec_interp_bin = spec_interp_dir.join("wasm");
        if !spec_interp_bin.exists() {
            println!("Spec interpreter not found at {:?}, trying to build it. (spoiler: ocaml / dune required)", spec_interp_bin);
            let result = std::process::Command::new("make")
                .current_dir(&spec_interp_dir)
                .output()
                .unwrap();
            if !result.status.success() {
                panic!("Failed to build spec interpreter: {:?}", result);
            }
        }

        let output = path.with_extension("bin.wast");
        let result = std::process::Command::new(spec_interp_bin)
            .arg(path)
            .arg("-o")
            .arg(output.clone())
            .output()
            .unwrap();
        if !result.status.success() {
            panic!("Failed to compile spec test: {:?}", result);
        }
        output
    }
}
