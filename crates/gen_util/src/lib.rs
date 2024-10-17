use core::panic;
use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use std::{fs, path::PathBuf};
use syn::{parse::Parse, Expr, Token};

struct GenMacroInput {
    test_function_name: Expr,
    _comma: Token![,],
    test_type_name: Expr,
}

impl Parse for GenMacroInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            test_function_name: input.parse()?,
            _comma: input.parse()?,
            test_type_name: input.parse()?,
        })
    }
}

#[proc_macro]
pub fn generate_spec_test_cases(input: TokenStream) -> TokenStream {
    let test_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("thirdparty/spec/test/core");
    if !test_dir.is_dir() {
        panic!("Spec test directory {} not found. Try running `git submodule update --init --recursive` to fetch the spec tests.", test_dir.display());
    }

    let mut test_cases = quote! {};
    let test_function = syn::parse_macro_input!(input as GenMacroInput);

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
                        let s = test_function
                            .test_type_name
                            .clone()
                            .into_token_stream()
                            .to_string();
                        let test_name =
                            format_ident!("spec_test_{}_{}", s, file_name.replace(['.', '-'], "_"));
                        let file_path = entry.path().to_str().unwrap().to_string();
                        let test_function_name = &test_function.test_function_name;
                        test_cases = quote! {
                            #test_cases

                            #[test_log::test]
                            fn #test_name() {
                                #test_function_name(#file_path);
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
