//! Expansion of the `#[boost_test::test]` attribute.

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    Expr, ExprLit, ItemFn, Lit, LitStr, Meta, Path, Signature, Type, Visibility, parse::Parser, punctuated::Punctuated, spanned::Spanned,
    token::Comma,
};

use super::{
    runtime::RuntimePath,
    signature::{is_context_type, is_mutable_reference, reject_unsupported_modifiers, typed_argument_type, unit_or_result},
};

/// Parsed `#[boost_test::test(...)]` attributes.
#[derive(Default)]
struct TestOptions {
    name: Option<String>,
    case: Option<LitStr>,
    fixture: Option<Path>,
    /// `Some(true)` means an expected failure, `Some(false)` means an expected pass.
    expected_failure: Option<bool>,
}

/// The declarative facts the expansion needs from the original signature.
struct SignatureShape {
    takes_context: bool,
    takes_fixture: bool,
    returns_result: bool,
}

/// Expands `#[boost_test::test]` into the original body plus a standard `#[test]` wrapper.
pub(crate) fn expand_test(attributes: TokenStream, item: ItemFn, default_runtime: RuntimePath) -> syn::Result<TokenStream> {
    let mut metas = if attributes.is_empty() {
        Punctuated::new()
    }
    else {
        Punctuated::<Meta, Comma>::parse_terminated.parse2(attributes)?
    };
    let (runtime, metas) = RuntimePath::take_from_metas(metas, default_runtime)?;
    let options = parse_options_from_metas(metas)?;
    let shape = validate_signature(&item.sig, options.fixture.is_some())?;

    let ident = item.sig.ident.clone();
    let body_ident = format_ident!("__testkit_body_{}", ident);
    let wrapper_visibility = item.vis.clone();
    let name = options.name.unwrap_or_else(|| ident.to_string());
    let name_literal = LitStr::new(&name, ident.span());
    let case_path = runtime.extend(&["case"]);
    let descriptor_path = runtime.extend(&["descriptor"]);
    let case_attachment = match &options.case {
        Some(case) => quote! {
            .with_case(#case_path::CaseId::new(#case).expect("`#[boost_test::test]` validates a non-empty case name"))
        },
        None => TokenStream::new(),
    };
    let expectation = if options.expected_failure.unwrap_or(false) {
        quote!(#descriptor_path::Expectation::ExpectedFailure)
    }
    else {
        quote!(#descriptor_path::Expectation::Pass)
    };

    let mut body = item;
    let mut wrapper_attributes = Vec::new();
    let mut body_attributes = Vec::new();
    for attribute in core::mem::take(&mut body.attrs) {
        let path = attribute.path();
        if path.is_ident("test") {
            continue;
        }
        if path.is_ident("should_panic") {
            return Err(syn::Error::new(
                attribute.span(),
                "`#[should_panic]` is not supported by `#[boost_test::test]`, use `expected = \"failure\"` instead",
            ));
        }
        if path.is_ident("ignore") {
            wrapper_attributes.push(attribute);
            continue;
        }
        if path.is_ident("cfg") || path.is_ident("cfg_attr") {
            wrapper_attributes.push(attribute.clone());
        }
        body_attributes.push(attribute);
    }
    body.attrs = body_attributes;
    body.vis = Visibility::Inherited;
    body.sig.ident = body_ident.clone();

    let context_ident = format_ident!("__testkit_context");
    let unused_context_ident = format_ident!("_testkit_context");
    let fixture_ident = format_ident!("__testkit_fixture");
    let (closure_parameters, body_call) = match (shape.takes_context, shape.takes_fixture) {
        (false, false) => (quote!(#unused_context_ident), quote!(#body_ident())),
        (true, false) => (quote!(#context_ident), quote!(#body_ident(#context_ident))),
        (false, true) => (quote!(#unused_context_ident, #fixture_ident), quote!(#body_ident(#fixture_ident))),
        (true, true) => (quote!(#context_ident, #fixture_ident), quote!(#body_ident(#context_ident, #fixture_ident))),
    };
    let body_expression = if shape.returns_result {
        body_call
    }
    else {
        quote!({
            #body_call;
            ::core::result::Result::<(), ::core::convert::Infallible>::Ok(())
        })
    };

    let run_with_fixture = runtime.extend(&["run_with_fixture"]);
    let run = runtime.extend(&["run"]);
    let descriptor = quote! {
        #descriptor_path::TestDescriptor::new(
            #name_literal,
            module_path!(),
            #descriptor_path::SourceLocation::new(file!(), line!(), column!()),
        )
        #case_attachment
        .with_expectation(#expectation)
    };
    let entry = match &options.fixture {
        Some(fixture) => quote! {
            #run_with_fixture::<#fixture, _, _>(
                #descriptor,
                |#closure_parameters| #body_expression,
            );
        },
        None => quote! {
            #run(
                #descriptor,
                |#closure_parameters| #body_expression,
            );
        },
    };

    let wrapper = quote! {
        #(#wrapper_attributes)*
        #[test]
        #wrapper_visibility fn #ident() {
            #entry
        }
    };

    Ok(quote! {
        #body
        #wrapper
    })
}

/// Parses the attribute arguments after `runtime = ...` has been removed.
fn parse_options_from_metas(metas: Punctuated<Meta, Comma>) -> syn::Result<TestOptions> {
    let mut options = TestOptions::default();
    for meta in metas {
        match meta {
            Meta::NameValue(name_value) if name_value.path.is_ident("name") => {
                if options.name.is_some() {
                    return Err(syn::Error::new(name_value.span(), "duplicate `name` attribute"));
                }
                let Expr::Lit(ExprLit { lit: Lit::Str(value), .. }) = name_value.value
                else {
                    return Err(syn::Error::new(name_value.span(), "`name` must be a string literal"));
                };
                options.name = Some(value.value());
            }
            Meta::NameValue(name_value) if name_value.path.is_ident("case") => {
                if options.case.is_some() {
                    return Err(syn::Error::new(name_value.span(), "duplicate `case` attribute"));
                }
                let Expr::Lit(ExprLit { lit: Lit::Str(value), .. }) = name_value.value
                else {
                    return Err(syn::Error::new(name_value.span(), "`case` must be a string literal"));
                };
                if value.value().is_empty() {
                    return Err(syn::Error::new(value.span(), "`case` must not be empty"));
                }
                options.case = Some(value);
            }
            Meta::NameValue(name_value) if name_value.path.is_ident("fixture") => {
                if options.fixture.is_some() {
                    return Err(syn::Error::new(name_value.span(), "duplicate `fixture` attribute"));
                }
                let Expr::Path(path) = name_value.value
                else {
                    return Err(syn::Error::new(name_value.span(), "`fixture` must name a fixture path"));
                };
                options.fixture = Some(path.path);
            }
            Meta::NameValue(name_value) if name_value.path.is_ident("expected") => {
                let span = name_value.span();
                let Expr::Lit(ExprLit { lit: Lit::Str(value), .. }) = name_value.value
                else {
                    return Err(syn::Error::new(span, "`expected` must be a string literal"));
                };
                let expected_failure = match value.value().as_str() {
                    "failure" => true,
                    "pass" => false,
                    _ => return Err(syn::Error::new(value.span(), "`expected` must be `\"pass\"` or `\"failure\"`")),
                };
                set_expectation(&mut options.expected_failure, expected_failure, span)?;
            }
            Meta::Path(path) if path.is_ident("expected_failure") => {
                set_expectation(&mut options.expected_failure, true, path.span())?;
            }
            other => {
                return Err(syn::Error::new(
                    other.span(),
                    "unsupported `#[boost_test::test]` attribute, supported attributes are `runtime`, `name`, `case`, `fixture`, `expected`, and `expected_failure`",
                ));
            }
        }
    }
    Ok(options)
}

/// Records one expectation declaration, rejecting a conflicting duplicate.
fn set_expectation(slot: &mut Option<bool>, expected_failure: bool, span: Span) -> syn::Result<()> {
    match slot {
        Some(current) if *current != expected_failure => Err(syn::Error::new(span, "conflicting `expected` declarations")),
        _ => {
            *slot = Some(expected_failure);
            Ok(())
        }
    }
}

/// Validates the signature and returns the facts needed by the wrapper.
fn validate_signature(signature: &Signature, has_fixture: bool) -> syn::Result<SignatureShape> {
    const ATTRIBUTE: &str = "#[boost_test::test]";
    reject_unsupported_modifiers(signature, ATTRIBUTE)?;

    let (takes_context, takes_fixture) = match (signature.inputs.len(), has_fixture) {
        (0, false) => (false, false),
        (1, false) => {
            let ty = typed_argument_type(&signature.inputs[0], ATTRIBUTE)?;
            if !is_context_type(ty) {
                return Err(syn::Error::new(ty.span(), format!("the argument of a `{ATTRIBUTE}` body must be `&mut boost_test::TestContext`")));
            }
            (true, false)
        }
        (1, true) => {
            let ty = typed_argument_type(&signature.inputs[0], ATTRIBUTE)?;
            if is_context_type(ty) {
                return Err(syn::Error::new(
                    ty.span(),
                    format!("a `{ATTRIBUTE}` body with `fixture` must also accept the fixture as `&mut Value`"),
                ));
            }
            ensure_fixture_argument(ty, ATTRIBUTE)?;
            (false, true)
        }
        (2, true) => {
            let context = typed_argument_type(&signature.inputs[0], ATTRIBUTE)?;
            let fixture = typed_argument_type(&signature.inputs[1], ATTRIBUTE)?;
            if !is_context_type(context) {
                return Err(syn::Error::new(
                    context.span(),
                    format!("the first argument of a `{ATTRIBUTE}` body with `fixture` must be `&mut boost_test::TestContext`"),
                ));
            }
            if is_context_type(fixture) {
                return Err(syn::Error::new(
                    fixture.span(),
                    format!("the second argument of a `{ATTRIBUTE}` body with `fixture` must be the fixture value"),
                ));
            }
            ensure_fixture_argument(fixture, ATTRIBUTE)?;
            (true, true)
        }
        (0, true) => {
            return Err(syn::Error::new(
                signature.inputs.span(),
                format!("a `{ATTRIBUTE}` body with `fixture` must accept the fixture as `&mut Value`"),
            ));
        }
        (_, true) => {
            return Err(syn::Error::new(
                signature.inputs.span(),
                format!("a `{ATTRIBUTE}` body with `fixture` takes the fixture as `&mut Value`, optionally after `&mut boost_test::TestContext`"),
            ));
        }
        _ => {
            return Err(syn::Error::new(
                signature.inputs.span(),
                format!("`{ATTRIBUTE}` bodies take at most one `&mut boost_test::TestContext` argument"),
            ));
        }
    };

    let returns_result = unit_or_result(&signature.output, ATTRIBUTE)?;
    Ok(SignatureShape { takes_context, takes_fixture, returns_result })
}

/// Requires the fixture argument to be a mutable reference.
fn ensure_fixture_argument(ty: &Type, attribute: &str) -> syn::Result<()> {
    if is_mutable_reference(ty) {
        Ok(())
    }
    else {
        Err(syn::Error::new(ty.span(), format!("the fixture argument of a `{attribute}` body must be `&mut Value`")))
    }
}
