//! A simple test case attribute macro that lets you write parameterised tests
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
    spanned::Spanned,
    Attribute, Error, Expr, FnArg, ItemFn, LitStr, PatType, Result, Stmt, Token,
};

struct TestCase {
    args: Punctuated<Expr, Token![,]>,
    name: LitStr,
    span: Span,
}

// #[test_case(1, 3.14, "foo"; "this is a test case")]
impl Parse for TestCase {
    fn parse(input: ParseStream<'_>) -> syn::parse::Result<Self> {
        let span = input.span();
        let args = Punctuated::parse_separated_nonempty_with(input, Expr::parse)?;
        let _: Token![;] = input.parse()?;
        let name: LitStr = input.parse()?;

        Ok(Self { args, name, span })
    }
}

/// Parameterise a test with multiple input values.
#[proc_macro_attribute]
pub fn test_case(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut original = parse_macro_input!(input as ItemFn);
    let first_case = parse_macro_input!(args as TestCase);

    // We should be the first test_case attribute, but there may be others beneath us so walk
    // through the attributes and parse any other test_cases we find
    let mut cases = vec![first_case];
    let to_remove = match extract_other_cases(&mut cases, &original.attrs) {
        Ok(indices) => indices,
        Err(e) => return TokenStream::from(e.into_compile_error()),
    };

    // Remove the other test_case attributes we've now parsed
    for i in to_remove.into_iter().rev() {
        original.attrs.swap_remove(i);
    }

    // For rendering we use the original function name as a module and snake_case convert the case
    // names we've been given to generate the new test case names.
    // Any existing attrs _other_ than ours are preserved and the original function is updated for
    // each case to bind the function parameters explicitly at the top;
    let module = original.sig.ident.clone();
    let resolved_cases = resolve_test_cases(original, cases);

    TokenStream::from(quote! {
        mod #module {
            #[allow(unused_imports)]
            use super::*;

            #(#resolved_cases)*
        }
    })
}

fn extract_other_cases(cases: &mut Vec<TestCase>, attrs: &[Attribute]) -> syn::Result<Vec<usize>> {
    let test_case_attr = parse_quote!(test_case);
    let qualified_test_case_attr = parse_quote!(simple_test_case::test_case);

    attrs
        .iter()
        .enumerate()
        .filter(|(_, a)| a.path == test_case_attr || a.path == qualified_test_case_attr)
        .map(|(ix, a)| match a.parse_args::<TestCase>() {
            Ok(test_case) => {
                cases.push(test_case);
                Ok(ix)
            }
            Err(err) => Err(syn::Error::new(
                a.span(),
                format!("invalid test_case: {}", err),
            )),
        })
        .collect()
}

fn resolve_test_cases(original: ItemFn, cases: Vec<TestCase>) -> Vec<proc_macro2::TokenStream> {
    cases
        .into_iter()
        .map(|c| resolve_test_case(original.clone(), c))
        .collect()
}

fn resolve_test_case(mut _fn: ItemFn, case: TestCase) -> proc_macro2::TokenStream {
    let TestCase { span, args, name } = case;

    // TODO: need to move the args from the sig into the new body Block
    _fn.sig.ident = slugify_ident(name);
    let inputs = _fn.sig.inputs.clone();
    _fn.sig.inputs.clear();

    if args.len() != inputs.len() {
        return Error::new(span, "wrong number of arguments").into_compile_error();
    }

    let res: Result<Vec<Stmt>> = inputs
        .iter()
        .zip(args)
        .map(|(fnarg, val)| match fnarg {
            FnArg::Typed(pt) => {
                let PatType { attrs, pat, ty, .. } = pt;
                if !attrs.is_empty() {
                    Err(Error::new_spanned(
                        fnarg,
                        "attributes on function arguments are not supported",
                    ))
                } else {
                    syn::parse2(quote! { let #pat: #ty = #val; })
                }
            }

            _ => Err(Error::new_spanned(
                fnarg,
                "self is not permitted for test cases",
            )),
        })
        .collect();

    match res {
        Ok(mut stmts) => {
            let as_written = _fn.block.stmts.clone();
            stmts.extend(as_written);
            _fn.block.stmts = stmts;
            _fn.into_token_stream()
        }
        Err(e) => e.into_compile_error(),
    }
}

// assume no non-alphanumeric and no leading digits
fn slugify_ident(name: LitStr) -> Ident {
    Ident::new(
        &name.value().to_ascii_lowercase().replace(' ', "_"),
        name.span(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{parse_quote, ExprLit, Lit};

    #[test]
    fn parse_test_case() {
        let input: proc_macro2::TokenStream = parse_quote! {
            "this", 1, true; "name here"
        };

        let parsed: TestCase = syn::parse2(input).unwrap();

        match &parsed.args[0] {
            Expr::Lit(ExprLit {
                lit: Lit::Str(s), ..
            }) => assert_eq!(s.value(), "this"),
            other => panic!("expected LitStr, got {:?}", other),
        }

        match &parsed.args[1] {
            Expr::Lit(ExprLit {
                lit: Lit::Int(n), ..
            }) => assert_eq!(n.base10_parse::<u8>().unwrap(), 1),
            other => panic!("expected LitInt, got {:?}", other),
        }

        match &parsed.args[2] {
            Expr::Lit(ExprLit {
                lit: Lit::Bool(b), ..
            }) => assert!(b.value),
            other => panic!("expected LitBool, got {:?}", other),
        }

        assert_eq!(parsed.name.value(), "name here");
    }
}
