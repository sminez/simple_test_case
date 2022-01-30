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

// A really simple test case specification of the form: test_case(exprs, ...; "name for test case")
// There is no defaulting of the case name or additional properties that can be set.
struct TestCase {
    args: Punctuated<Expr, Token![,]>,
    name: LitStr,
    span: Span,
}

impl Parse for TestCase {
    fn parse(input: ParseStream<'_>) -> syn::parse::Result<Self> {
        let span = input.span();
        let args = Punctuated::parse_separated_nonempty_with(input, Expr::parse)?;
        let _: Token![;] = input.parse()?;
        let name: LitStr = input.parse()?;

        Ok(Self { args, name, span })
    }
}

pub(crate) fn inner(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut original = parse_macro_input!(input as ItemFn);
    let first_case = parse_macro_input!(args as TestCase);
    let module = original.sig.ident.clone();

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

    let resolved_cases: Vec<_> = cases
        .into_iter()
        .map(|c| resolve_test_case(original.clone(), c))
        .collect();

    TokenStream::from(quote! {
        mod #module {
            #[allow(unused_imports)]
            use super::*;

            #(#resolved_cases)*
        }
    })
}

// Glob up any other `test_case` attribute macros underneath us and parse them as additional
// cases that we will handle generating.
fn extract_other_cases(cases: &mut Vec<TestCase>, attrs: &[Attribute]) -> Result<Vec<usize>> {
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
            Err(err) => Err(Error::new(a.span(), format!("invalid test_case: {}", err))),
        })
        .collect()
}

// For rendering we use the original function name as a module and snake_case convert the case
// names we've been given to generate the new test case names. Any existing attrs _other_ than ours
// are preserved and the original function is updated for each case to bind the function parameters
// explicitly at the top.
fn resolve_test_case(mut _fn: ItemFn, case: TestCase) -> proc_macro2::TokenStream {
    let TestCase { span, args, name } = case;
    let inputs = _fn.sig.inputs.clone();

    // Explicitly bail on mismatched number of arguments rather than silently dropping from the
    // shorter side of the `zip` used for generating the variable bindings.
    if args.len() != inputs.len() {
        return Error::new(span, "wrong number of arguments").into_compile_error();
    }

    // Strip the original function arguments so that `_fn` will be valid as a test function
    _fn.sig.inputs.clear();

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

            FnArg::Receiver(_) => Err(Error::new_spanned(
                fnarg,
                "self is not permitted for test cases",
            )),
        })
        .collect();

    match res {
        // Add variable bindings (in place of function parameters) to the top of the function body
        // and set the name of this test case to be the one we were given
        Ok(mut stmts) => {
            let as_written = _fn.block.stmts.clone();
            stmts.extend(as_written);
            _fn.block.stmts = stmts;
            _fn.sig.ident = slugify_ident(name);

            _fn.into_token_stream()
        }

        // Something was invalid (in terms of what we support) about the original function args so
        // report the error and bail
        Err(e) => e.into_compile_error(),
    }
}

// assumes no non-alphanumeric characters and no leading digits
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
