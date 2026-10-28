//! Configurable runtime crate path for generated test wrappers.

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Expr, Meta, Path, punctuated::Punctuated, spanned::Spanned, token::Comma};

/// Absolute path prefix for `boost-test` runtime items in generated code.
#[derive(Clone)]
pub(crate) struct RuntimePath(Path);

impl RuntimePath {
    pub(crate) fn direct() -> Self {
        Self(syn::parse_str("::boost_test").expect("direct runtime path"))
    }

    pub(crate) fn facade() -> Self {
        Self(syn::parse_str("::boost::test").expect("facade runtime path"))
    }

    pub(crate) fn from_path(path: Path) -> Self {
        Self(path)
    }

    pub(crate) fn extend(&self, segments: &[&str]) -> Path {
        let mut path = self.0.clone();
        for segment in segments {
            path.segments.push(syn::PathSegment::from(syn::Ident::new(segment, Span::call_site())));
        }
        path
    }

    pub(crate) fn tokens(&self) -> TokenStream {
        let path = &self.0;
        quote!(#path)
    }

    /// Removes `runtime = path` from attribute metas when present and returns the resolved path.
    pub(crate) fn take_from_metas(metas: Punctuated<Meta, Comma>, default: Self) -> syn::Result<(Self, Punctuated<Meta, Comma>)> {
        let mut runtime = default;
        let mut remaining = Punctuated::new();
        for meta in metas {
            if let Meta::NameValue(name_value) = &meta {
                if name_value.path.is_ident("runtime") {
                    let Expr::Path(expr_path) = &name_value.value
                    else {
                        return Err(syn::Error::new(
                            name_value.span(),
                            "`runtime` must be an absolute path such as `::boost::test` or `::my_test`",
                        ));
                    };
                    runtime = Self::from_path(expr_path.path.clone());
                    continue;
                }
            }
            remaining.push(meta);
        }
        Ok((runtime, remaining))
    }
}
