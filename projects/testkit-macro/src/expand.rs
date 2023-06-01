//! Expansion of the `#[testkit::test]` attribute.

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    Expr, ExprLit, FnArg, GenericArgument, ItemFn, Lit, LitStr, Meta, PathArguments, ReturnType, Signature, Type, Visibility, parse::Parser,
    punctuated::Punctuated, spanned::Spanned, token::Comma,
};

/// Parsed `#[testkit::test(...)]` attributes.
#[derive(Default)]
struct TestOptions {
    name: Option<String>,
    /// `Some(true)` means an expected failure, `Some(false)` means an expected pass.
    expected_failure: Option<bool>,
}

/// The two declarative facts the expansion needs from the original signature.
struct SignatureShape {
    takes_context: bool,
    returns_result: bool,
}

/// Expands `#[testkit::test]` into the original body plus a standard `#[test]` wrapper.
pub(crate) fn expand_test(attributes: TokenStream, item: ItemFn) -> syn::Result<TokenStream> {
    let options = parse_options(attributes)?;
    let shape = validate_signature(&item.sig)?;

    let ident = item.sig.ident.clone();
    let body_ident = format_ident!("__testkit_body_{}", ident);
    let wrapper_visibility = item.vis.clone();
    let name = options.name.unwrap_or_else(|| ident.to_string());
    let name_literal = LitStr::new(&name, ident.span());
    let expectation = if options.expected_failure.unwrap_or(false) {
        quote!(::testkit::descriptor::Expectation::ExpectedFailure)
    }
    else {
        quote!(::testkit::descriptor::Expectation::Pass)
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
                "`#[should_panic]` is not supported by `#[testkit::test]`, use `expected = \"failure\"` instead",
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
    let (closure_parameter, body_call) = if shape.takes_context {
        (quote!(#context_ident), quote!(#body_ident(#context_ident)))
    }
    else {
        (quote!(#unused_context_ident), quote!(#body_ident()))
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

    let wrapper = quote! {
        #(#wrapper_attributes)*
        #[test]
        #wrapper_visibility fn #ident() {
            ::testkit::run(
                ::testkit::descriptor::TestDescriptor::new(
                    #name_literal,
                    module_path!(),
                    ::testkit::descriptor::SourceLocation::new(file!(), line!(), column!()),
                )
                .with_expectation(#expectation),
                |#closure_parameter| #body_expression,
            );
        }
    };

    Ok(quote! {
        #body
        #wrapper
    })
}

/// Parses the attribute arguments.
fn parse_options(attributes: TokenStream) -> syn::Result<TestOptions> {
    if attributes.is_empty() {
        return Ok(TestOptions::default());
    }
    let parser = Punctuated::<Meta, Comma>::parse_terminated;
    let metas = parser.parse2(attributes)?;
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
                    "unsupported `#[testkit::test]` attribute, supported attributes are `name`, `expected`, and `expected_failure`",
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
fn validate_signature(signature: &Signature) -> syn::Result<SignatureShape> {
    if let Some(asyncness) = signature.asyncness {
        return Err(syn::Error::new(asyncness.span, "`#[testkit::test]` does not support `async` functions yet"));
    }
    if let Some(constness) = signature.constness {
        return Err(syn::Error::new(constness.span, "`#[testkit::test]` does not support `const` functions"));
    }
    if let Some(unsafety) = signature.unsafety {
        return Err(syn::Error::new(unsafety.span, "`#[testkit::test]` does not support `unsafe` functions"));
    }
    if let Some(abi) = &signature.abi {
        return Err(syn::Error::new(abi.span(), "`#[testkit::test]` does not support `extern` functions"));
    }
    if !signature.generics.params.is_empty() || signature.generics.where_clause.is_some() {
        return Err(syn::Error::new(signature.generics.span(), "`#[testkit::test]` does not support generic functions"));
    }
    if let Some(variadic) = &signature.variadic {
        return Err(syn::Error::new(variadic.span(), "`#[testkit::test]` does not support variadic functions"));
    }

    let takes_context = match signature.inputs.len() {
        0 => false,
        1 => match &signature.inputs[0] {
            FnArg::Receiver(receiver) => {
                return Err(syn::Error::new(receiver.span(), "`#[testkit::test]` supports free functions, not methods with a receiver"));
            }
            FnArg::Typed(argument) => {
                if !is_context_type(&argument.ty) {
                    return Err(syn::Error::new(
                        argument.ty.span(),
                        "the argument of a `#[testkit::test]` body must be `&mut testkit::TestContext`",
                    ));
                }
                true
            }
        },
        _ => {
            return Err(syn::Error::new(
                signature.inputs.span(),
                "`#[testkit::test]` bodies take at most one `&mut testkit::TestContext` argument",
            ));
        }
    };

    let returns_result = validate_return(&signature.output)?;
    Ok(SignatureShape { takes_context, returns_result })
}

/// Returns whether the type is `&mut ...::TestContext`.
fn is_context_type(ty: &Type) -> bool {
    let Type::Reference(reference) = ty
    else {
        return false;
    };
    if reference.mutability.is_none() {
        return false;
    }
    let Type::Path(path) = &*reference.elem
    else {
        return false;
    };
    path.path.segments.last().is_some_and(|segment| segment.ident == "TestContext")
}

/// Validates the return type and reports whether the body returns a `Result`.
fn validate_return(output: &ReturnType) -> syn::Result<bool> {
    let ReturnType::Type(_, ty) = output
    else {
        return Ok(false);
    };
    let Type::Path(path) = &**ty
    else {
        return Err(syn::Error::new(ty.span(), "a `#[testkit::test]` body must return `()` or `Result<(), E>`"));
    };
    let Some(segment) = path.path.segments.last()
    else {
        return Err(syn::Error::new(ty.span(), "a `#[testkit::test]` body must return `()` or `Result<(), E>`"));
    };
    if segment.ident != "Result" {
        return Err(syn::Error::new(ty.span(), "a `#[testkit::test]` body must return `()` or `Result<(), E>`"));
    }
    let PathArguments::AngleBracketed(arguments) = &segment.arguments
    else {
        return Err(syn::Error::new(ty.span(), "a returned `Result` must name both `()` and the error type"));
    };
    if arguments.args.len() != 2 {
        return Err(syn::Error::new(ty.span(), "a returned `Result` must name both `()` and the error type"));
    }
    let Some(GenericArgument::Type(first)) = arguments.args.first()
    else {
        return Err(syn::Error::new(ty.span(), "a returned `Result` must use `()` as its success type"));
    };
    if !matches!(first, Type::Tuple(tuple) if tuple.elems.is_empty()) {
        return Err(syn::Error::new(first.span(), "a `#[testkit::test]` body must return `Result<(), E>`, not a value"));
    }
    Ok(true)
}
