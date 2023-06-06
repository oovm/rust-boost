//! Expansion of the `#[boost_test::case]` attribute.

use std::collections::HashSet;

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    Expr, Ident, ItemFn, LitStr, Token, Visibility,
    parse::{Parse, ParseStream},
    spanned::Spanned,
};

use super::signature::{reject_receiver, reject_unsupported_modifiers, unit_or_result};

/// One declared case: a stable name and the expression that supplies its arguments.
struct CaseEntry {
    name: LitStr,
    arguments: Expr,
}

/// The complete parameter table.
struct CaseEntries(Vec<CaseEntry>);

impl Parse for CaseEntries {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let mut entries = Vec::new();
        while !input.is_empty() {
            let name: LitStr = input.parse()?;
            input.parse::<Token![=>]>()?;
            let arguments: Expr = input.parse()?;
            entries.push(CaseEntry { name, arguments });
            if input.is_empty() {
                break;
            }
            input.parse::<Token![,]>()?;
        }
        Ok(Self(entries))
    }
}

/// Expands `#[boost_test::case]` into one standard `#[test]` wrapper per declared case.
pub(crate) fn expand_case(attributes: TokenStream, item: ItemFn) -> syn::Result<TokenStream> {
    const ATTRIBUTE: &str = "#[boost_test::case]";
    let entries = syn::parse2::<CaseEntries>(attributes)?.0;
    if entries.is_empty() {
        return Err(syn::Error::new(Span::call_site(), "`#[boost_test::case]` needs at least one `\"name\" => arguments` entry"));
    }
    reject_unsupported_modifiers(&item.sig, ATTRIBUTE)?;
    reject_receiver(&item.sig, ATTRIBUTE)?;
    if item.sig.inputs.is_empty() {
        return Err(syn::Error::new(
            item.sig.inputs.span(),
            "`#[boost_test::case]` needs at least one parameter, use `#[boost_test::test]` for a parameterless entry",
        ));
    }
    let parameter_count = item.sig.inputs.len();
    let returns_result = unit_or_result(&item.sig.output, ATTRIBUTE)?;

    let ident = item.sig.ident.clone();
    let body_ident = format_ident!("__testkit_case_body_{}", ident);
    let name_literal = LitStr::new(&ident.to_string(), ident.span());

    let mut body = item;
    let mut shared_attributes = Vec::new();
    let mut body_attributes = Vec::new();
    for attribute in core::mem::take(&mut body.attrs) {
        let path = attribute.path();
        if path.is_ident("test") {
            continue;
        }
        if path.is_ident("should_panic") {
            return Err(syn::Error::new(
                attribute.span(),
                "`#[should_panic]` is not supported by `#[boost_test::case]`, declare an expected failure per case instead",
            ));
        }
        if path.is_ident("ignore") || path.is_ident("cfg") || path.is_ident("cfg_attr") {
            shared_attributes.push(attribute.clone());
        }
        body_attributes.push(attribute);
    }
    body.attrs = body_attributes;
    body.vis = Visibility::Inherited;
    body.sig.ident = body_ident.clone();

    let mut names = HashSet::new();
    let mut wrapper_idents = HashSet::new();
    let mut wrappers = Vec::with_capacity(entries.len());
    for entry in entries {
        let case_name = entry.name.value();
        if case_name.is_empty() {
            return Err(syn::Error::new(entry.name.span(), "a case name must not be empty"));
        }
        if !names.insert(case_name.clone()) {
            return Err(syn::Error::new(entry.name.span(), format!("duplicate case name `{case_name}`")));
        }
        let wrapper_ident = case_wrapper_ident(&ident, &case_name);
        if !wrapper_idents.insert(wrapper_ident.to_string()) {
            return Err(syn::Error::new(
                entry.name.span(),
                format!("the case name `{case_name}` produces a test function that collides with another case, choose a more distinct name"),
            ));
        }
        let call_arguments = case_call_arguments(&entry.arguments, parameter_count, &case_name)?;
        let body_expression = if returns_result {
            quote!(#body_ident(#call_arguments))
        }
        else {
            quote!({
                #body_ident(#call_arguments);
                ::core::result::Result::<(), ::core::convert::Infallible>::Ok(())
            })
        };
        let case_literal = &entry.name;
        wrappers.push(quote! {
            #(#shared_attributes)*
            #[allow(non_snake_case)]
            #[test]
            fn #wrapper_ident() {
                ::boost_test::run(
                    ::boost_test::descriptor::TestDescriptor::new(
                        #name_literal,
                        module_path!(),
                        ::boost_test::descriptor::SourceLocation::new(file!(), line!(), column!()),
                    )
                    .with_case(::boost_test::case::CaseId::new(#case_literal).expect("`#[boost_test::case]` validates non-empty names")),
                    |_testkit_context| #body_expression,
                );
            }
        });
    }

    Ok(quote! {
        #body
        #(#wrappers)*
    })
}

/// Builds the test function name from the body name and a sanitized case name.
fn case_wrapper_ident(body_ident: &Ident, case_name: &str) -> Ident {
    let mut sanitized = String::with_capacity(case_name.len());
    for character in case_name.chars() {
        if character.is_ascii_alphanumeric() { sanitized.push(character) } else { sanitized.push('_') }
    }
    if sanitized.is_empty() {
        sanitized.push('_');
    }
    if sanitized.as_bytes().first().is_some_and(|byte| byte.is_ascii_digit()) {
        sanitized.insert(0, '_');
    }
    format_ident!("{}__{}", body_ident, sanitized)
}

/// Extracts the positional arguments of one case and checks them against the body signature.
fn case_call_arguments(arguments: &Expr, parameter_count: usize, case_name: &str) -> syn::Result<TokenStream> {
    if parameter_count == 1 {
        return Ok(quote!(#arguments));
    }
    let Expr::Tuple(tuple) = arguments
    else {
        return Err(syn::Error::new(arguments.span(), format!("the case `{case_name}` must provide a tuple of {parameter_count} arguments")));
    };
    if tuple.elems.len() != parameter_count {
        return Err(syn::Error::new(
            tuple.span(),
            format!("the case `{case_name}` provides {} arguments but the function takes {parameter_count}", tuple.elems.len()),
        ));
    }
    let elements = tuple.elems.iter();
    Ok(quote!(#(#elements),*))
}
