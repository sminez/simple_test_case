use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use std::fs::{read_dir, read_to_string};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote, Error, FnArg, ItemFn, LitStr, Type,
};

struct DirCases {
    span: Span,
    dir: String,
}

impl Parse for DirCases {
    fn parse(input: ParseStream<'_>) -> syn::parse::Result<Self> {
        let span = input.span();
        let dir: LitStr = input.parse()?;

        Ok(Self {
            span,
            dir: dir.value(),
        })
    }
}

fn get_cases(dir: &str) -> Result<Vec<(String, String, String)>, std::io::Error> {
    let mut cases = vec![];

    for entry in read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let contents = read_to_string(path)?;
            let fname = entry.file_name().into_string().unwrap();
            let case = slugify_path(&fname);
            cases.push((format!("{}/{}", dir, fname), case, contents));
        }
    }

    Ok(cases)
}

fn slugify_path(p: &str) -> String {
    p.to_ascii_lowercase()
        .replace(' ', "_")
        .replace('.', "_")
        .replace('-', "_")
        .replace('/', "_")
}

fn has_correct_args(_fn: &ItemFn) -> bool {
    let str_ty: Type = parse_quote!(&str);
    let valid = |fnarg: &FnArg| matches!(fnarg, FnArg::Typed(pt) if *pt.ty == str_ty);

    _fn.sig.inputs.len() == 2 && _fn.sig.inputs.iter().all(valid)
}

pub(crate) fn inner(args: TokenStream, input: TokenStream) -> TokenStream {
    let DirCases { span, dir } = parse_macro_input!(args as DirCases);
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

    let case_details = match get_cases(&dir) {
        Ok(details) => details,
        Err(e) => {
            return TokenStream::from(
                Error::new(span, format!("Error loading test cases: {}", e)).into_compile_error(),
            )
        }
    };

    let case_attrs: Vec<_> = case_details
        .into_iter()
        .map(|(path, case, contents)| {
            quote! {
                #[simple_test_case::test_case(#path, #contents; #case)]
            }
        })
        .collect();

    TokenStream::from(quote! {
        #(#case_attrs)*
        #original
    })
}
