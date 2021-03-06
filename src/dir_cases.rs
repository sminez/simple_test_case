use crate::util::slugify_path;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use std::fs::read_dir;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
    Error, FnArg, ItemFn, LitStr, Token, Type,
};

struct DirCases {
    span: Span,
    dirs: Vec<String>,
}

impl Parse for DirCases {
    fn parse(input: ParseStream<'_>) -> syn::parse::Result<Self> {
        let span = input.span();
        let dirs: Punctuated<LitStr, Token![,]> = Punctuated::parse_separated_nonempty(input)?;
        let dirs: Vec<String> = dirs.iter().map(|d| d.value()).collect();

        Ok(Self { span, dirs })
    }
}

fn get_cases(dir: &str) -> Result<Vec<(String, String, String)>, std::io::Error> {
    let mut cases = vec![];
    let root = std::env::current_dir()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    for entry in read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let fname = entry.file_name().into_string().unwrap();
            let case = slugify_path(&format!("{}/{}", dir, fname));

            cases.push((
                format!("{}/{}", dir, fname),
                format!("{}/{}/{}", root, dir, fname),
                case,
            ));
        }
    }

    Ok(cases)
}

fn has_correct_args(_fn: &ItemFn) -> bool {
    let str_ty: Type = parse_quote!(&str);
    let valid = |fnarg: &FnArg| matches!(fnarg, FnArg::Typed(pt) if *pt.ty == str_ty);

    _fn.sig.inputs.len() == 2 && _fn.sig.inputs.iter().all(valid)
}

pub(crate) fn inner(args: TokenStream, input: TokenStream) -> TokenStream {
    let DirCases { span, dirs } = parse_macro_input!(args as DirCases);
    let original = parse_macro_input!(input as ItemFn);

    if !has_correct_args(&original) {
        return TokenStream::from(
            Error::new(
                span,
                "dir_cases test functions must accept (path: &str, contents: &str) as arguments"
                    .to_string(),
            )
            .into_compile_error(),
        );
    }

    let mut case_details = Vec::new();

    for dir in dirs.iter() {
        match get_cases(dir) {
            Ok(details) => case_details.extend(details),
            Err(e) => {
                return TokenStream::from(
                    Error::new(span, format!("Error loading test cases: {}", e))
                        .into_compile_error(),
                )
            }
        };
    }

    let case_attrs: Vec<_> = case_details
        .into_iter()
        .map(|(path, abs_path, case)| {
            quote! {
                #[simple_test_case::test_case(#path, include_str!(#abs_path); #case)]
            }
        })
        .collect();

    TokenStream::from(quote! {
        #(#case_attrs)*
        #original
    })
}
