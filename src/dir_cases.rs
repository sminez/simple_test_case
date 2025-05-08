use crate::util::slugify_path;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use std::{
    collections::HashMap,
    env::current_dir,
    fs::read_dir,
    io,
    path::{Path, PathBuf},
};
use syn::{
    parse::{self, Parse, ParseStream},
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
    Error, FnArg, ItemFn, LitStr, Token, Type,
};

const DUPLICATE_CASES_ERROR: &str = "\
When using dir_cases with multiple directories you must ensure that all file
names within the specified directories are unique.

The following test cases are defined multiple times,
";

struct DirCases {
    span: Span,
    dirs: Vec<String>,
}

impl Parse for DirCases {
    fn parse(input: ParseStream<'_>) -> parse::Result<Self> {
        let span = input.span();
        let dirs: Punctuated<LitStr, Token![,]> = Punctuated::parse_separated_nonempty(input)?;
        let dirs: Vec<String> = dirs.iter().map(|d| d.value()).collect();

        Ok(Self { span, dirs })
    }
}

fn get_cases(str_dir: &str, root: &Path) -> Result<Vec<(String, String, String)>, io::Error> {
    let dir = PathBuf::from(str_dir);
    if !dir.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("{str_dir} is not a known directory"),
        ));
    }

    let mut cases = Vec::new();

    for entry in read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let os_fname = entry.file_name();
            let fname = os_fname.to_string_lossy();
            let case = if entry.path().extension().is_some() {
                let (without_ext, _) = fname
                    .rsplit_once('.')
                    .expect("extension was Some so we have a dot");
                slugify_path(without_ext)
            } else {
                slugify_path(&fname)
            };

            let rel_path = dir.join(fname.as_ref());
            cases.push((
                rel_path.display().to_string(),
                root.join(rel_path).display().to_string(),
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
                "dir_cases test functions must accept (path: &str, contents: &str) as arguments",
            )
            .into_compile_error(),
        );
    }

    let root = match current_dir() {
        Ok(root) => root,
        Err(e) => {
            return TokenStream::from(
                Error::new(span, format!("Unable to determine working directory: {e}"))
                    .into_compile_error(),
            );
        }
    };
    let mut case_details = Vec::new();

    for dir in dirs.iter() {
        match get_cases(dir, &root) {
            Ok(details) => case_details.extend(details),
            Err(e) => {
                return TokenStream::from(
                    Error::new(span, format!("Error loading test cases: {e}")).into_compile_error(),
                )
            }
        };
    }

    // Try to give a nicer error message around duplicated test case names when the user provided
    // multiple directories and the same file name was present more than once.
    let mut seen: HashMap<&String, Vec<&String>> = HashMap::new();
    for (rel_path, _, case) in case_details.iter() {
        seen.entry(case).or_default().push(rel_path);
    }
    seen.retain(|_, paths| paths.len() > 1);
    if !seen.is_empty() {
        let duplicate_cases: Vec<String> = seen
            .into_iter()
            .map(|(case, rel_paths)| {
                let paths: Vec<String> = rel_paths.into_iter().map(|s| format!("  {s}")).collect();
                format!("{case}:\n{}", paths.join("\n"))
            })
            .collect();

        return TokenStream::from(
            Error::new(
                span,
                format!("{DUPLICATE_CASES_ERROR}\n{}", duplicate_cases.join("\n\n")),
            )
            .into_compile_error(),
        );
    }

    // If we're all good, write out the test cases by deferring to the test_case macro
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
