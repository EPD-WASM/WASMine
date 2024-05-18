use core::panic;
use std::{fs, path::PathBuf};

use proc_macro::TokenStream;
use quote::{format_ident, quote};

#[proc_macro]
pub fn generate_spec_test_cases(input: TokenStream) -> TokenStream {
    let test_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("thirdparty/spec/test/core");
    if !test_dir.is_dir() {
        panic!("Spec test directory {} not found. Try running `git submodule update --init --recursive` to fetch the spec tests.", test_dir.display());
    }

    let mut test_cases = quote! {};
    let test_function = syn::parse_macro_input!(input as syn::Expr);

    // Iterate over files in the test directory
    if let Ok(entries) = fs::read_dir(test_dir) {
        for entry in entries {
            match entry {
                Ok(entry)
                    if entry
                        .path()
                        .extension()
                        .map(|e| e == "wast")
                        .unwrap_or_default() =>
                {
                    if let Some(file_name) = entry.file_name().to_str() {
                        let test_name =
                            format_ident!("spec_test_{}", file_name.replace(['.', '-'], "_"));
                        let file_path = entry.path().to_str().unwrap().to_string();
                        test_cases = quote! {
                            #test_cases

                            #[test]
                            fn #test_name() {
                                #test_function(#file_path);
                            }
                        };
                    }
                }
                Err(e) => {
                    panic!("Error reading file: {:?}", e)
                }
                _ => (),
            }
        }
    }

    test_cases.into()
}
