//! Shared signature validation for the `testkit` attributes.

use syn::{FnArg, GenericArgument, PathArguments, ReturnType, Signature, Type, spanned::Spanned};

/// Rejects the function modifiers that no generated wrapper preserves.
pub(crate) fn reject_unsupported_modifiers(signature: &Signature, attribute: &str) -> syn::Result<()> {
    if let Some(asyncness) = signature.asyncness {
        return Err(syn::Error::new(asyncness.span, format!("`{attribute}` does not support `async` functions yet")));
    }
    if let Some(constness) = signature.constness {
        return Err(syn::Error::new(constness.span, format!("`{attribute}` does not support `const` functions")));
    }
    if let Some(unsafety) = signature.unsafety {
        return Err(syn::Error::new(unsafety.span, format!("`{attribute}` does not support `unsafe` functions")));
    }
    if let Some(abi) = &signature.abi {
        return Err(syn::Error::new(abi.span(), format!("`{attribute}` does not support `extern` functions")));
    }
    if !signature.generics.params.is_empty() || signature.generics.where_clause.is_some() {
        return Err(syn::Error::new(signature.generics.span(), format!("`{attribute}` does not support generic functions")));
    }
    if let Some(variadic) = &signature.variadic {
        return Err(syn::Error::new(variadic.span(), format!("`{attribute}` does not support variadic functions")));
    }
    Ok(())
}

/// Rejects a method receiver with a span on the receiver itself.
pub(crate) fn reject_receiver(signature: &Signature, attribute: &str) -> syn::Result<()> {
    for argument in &signature.inputs {
        if let FnArg::Receiver(receiver) = argument {
            return Err(syn::Error::new(receiver.span(), format!("`{attribute}` supports free functions, not methods with a receiver")));
        }
    }
    Ok(())
}

/// Returns the declared type of a typed argument.
pub(crate) fn typed_argument_type<'a>(argument: &'a FnArg, attribute: &str) -> syn::Result<&'a Type> {
    match argument {
        FnArg::Receiver(receiver) => {
            Err(syn::Error::new(receiver.span(), format!("`{attribute}` supports free functions, not methods with a receiver")))
        }
        FnArg::Typed(argument) => Ok(&argument.ty),
    }
}

/// Returns whether the type is `&mut ...::TestContext`.
pub(crate) fn is_context_type(ty: &Type) -> bool {
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

/// Returns whether the type is a mutable reference.
pub(crate) fn is_mutable_reference(ty: &Type) -> bool {
    matches!(ty, Type::Reference(reference) if reference.mutability.is_some())
}

/// Validates that a return is `()` or `Result<(), E>` and reports whether a `Result` is declared.
pub(crate) fn unit_or_result(output: &ReturnType, attribute: &str) -> syn::Result<bool> {
    let ReturnType::Type(_, ty) = output
    else {
        return Ok(false);
    };
    let Type::Path(path) = &**ty
    else {
        return Err(syn::Error::new(ty.span(), format!("a `{attribute}` body must return `()` or `Result<(), E>`")));
    };
    let Some(segment) = path.path.segments.last()
    else {
        return Err(syn::Error::new(ty.span(), format!("a `{attribute}` body must return `()` or `Result<(), E>`")));
    };
    if segment.ident != "Result" {
        return Err(syn::Error::new(ty.span(), format!("a `{attribute}` body must return `()` or `Result<(), E>`")));
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
        return Err(syn::Error::new(first.span(), format!("a `{attribute}` body must return `Result<(), E>`, not a value")));
    }
    Ok(true)
}

/// The declared return shape of a fixture setup function.
pub(crate) enum FixtureReturn {
    /// The function returns the fixture value directly.
    Value(Type),
    /// The function returns `Result<Value, E>`.
    Result { value: Type },
}

/// Classifies a fixture setup return as a plain value or a `Result<Value, E>`.
pub(crate) fn fixture_return(output: &ReturnType, attribute: &str) -> syn::Result<FixtureReturn> {
    let ReturnType::Type(_, ty) = output
    else {
        return Ok(FixtureReturn::Value(syn::parse_quote!(())));
    };
    if let Type::Path(path) = &**ty
        && let Some(segment) = path.path.segments.last()
        && segment.ident == "Result"
    {
        let PathArguments::AngleBracketed(arguments) = &segment.arguments
        else {
            return Err(syn::Error::new(ty.span(), format!("a `{attribute}` setup must return a value or `Result<Value, E>`")));
        };
        if arguments.args.len() != 2 {
            return Err(syn::Error::new(ty.span(), format!("a `{attribute}` setup must return a value or `Result<Value, E>`")));
        }
        let Some(GenericArgument::Type(value)) = arguments.args.first()
        else {
            return Err(syn::Error::new(ty.span(), "the `Result` of a fixture setup must name its success type"));
        };
        if !matches!(arguments.args.iter().nth(1), Some(GenericArgument::Type(_))) {
            return Err(syn::Error::new(ty.span(), "the `Result` of a fixture setup must name its error type"));
        }
        return Ok(FixtureReturn::Result { value: value.clone() });
    }
    Ok(FixtureReturn::Value((**ty).clone()))
}
