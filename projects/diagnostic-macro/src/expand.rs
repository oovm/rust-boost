use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    Ident, LitStr, Result, Token,
};

pub struct DiagnosticInput {
    pub code: LitStr,
    pub severity: Severity,
    pub message: MessageSpec,
    pub origin: Option<OriginSpec>,
}

pub enum Severity {
    Bug,
    Error,
    Warning,
    Info,
    Hint,
}

pub enum MessageSpec {
    Text(LitStr),
    Structured {
        key: LitStr,
        fallback: Option<LitStr>,
    },
}

pub struct OriginSpec {
    pub namespace: LitStr,
    pub component: LitStr,
    pub stage: Option<LitStr>,
}

impl Parse for DiagnosticInput {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut code = None;
        let mut severity = None;
        let mut message = None;
        let mut origin = None;

        while !input.is_empty() {
            let name: Ident = input.parse()?;
            input.parse::<Token![:]>()?;

            match name.to_string().as_str() {
                "code" => code = Some(input.parse()?),
                "severity" => severity = Some(parse_severity(input)?),
                "message" => message = Some(parse_message(input)?),
                "origin" => origin = Some(parse_origin(input)?),
                other => {
                    return Err(syn::Error::new(
                        name.span(),
                        format!("unknown field `{other}`; expected `code`, `severity`, `message`, or `origin`"),
                    ));
                }
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self {
            code: code.ok_or_else(|| syn::Error::new(input.span(), "missing field `code`"))?,
            severity: severity.ok_or_else(|| syn::Error::new(input.span(), "missing field `severity`"))?,
            message: message.ok_or_else(|| syn::Error::new(input.span(), "missing field `message`"))?,
            origin,
        })
    }
}

fn parse_severity(input: ParseStream) -> Result<Severity> {
    let ident: Ident = input.parse()?;
    match ident.to_string().as_str() {
        "bug" => Ok(Severity::Bug),
        "error" => Ok(Severity::Error),
        "warning" => Ok(Severity::Warning),
        "info" => Ok(Severity::Info),
        "hint" => Ok(Severity::Hint),
        other => Err(syn::Error::new(ident.span(), format!("unknown severity `{other}`"))),
    }
}

fn parse_message(input: ParseStream) -> Result<MessageSpec> {
    if input.peek(LitStr) {
        return Ok(MessageSpec::Text(input.parse()?));
    }

    let content;
    syn::parenthesized!(content in input);

    let mut key = None;
    let mut fallback = None;

    while !content.is_empty() {
        let name: Ident = content.parse()?;
        content.parse::<Token![:]>()?;
        match name.to_string().as_str() {
            "key" => key = Some(content.parse()?),
            "fallback" => fallback = Some(content.parse()?),
            other => {
                return Err(syn::Error::new(
                    name.span(),
                    format!("unknown message field `{other}`; expected `key` or `fallback`"),
                ));
            }
        }
        if content.peek(Token![,]) {
            content.parse::<Token![,]>()?;
        }
    }

    Ok(MessageSpec::Structured {
        key: key.ok_or_else(|| syn::Error::new(input.span(), "missing message field `key`"))?,
        fallback,
    })
}

fn parse_origin(input: ParseStream) -> Result<OriginSpec> {
    let content;
    syn::parenthesized!(content in input);

    let mut namespace = None;
    let mut component = None;
    let mut stage = None;

    while !content.is_empty() {
        let name: Ident = content.parse()?;
        content.parse::<Token![:]>()?;
        match name.to_string().as_str() {
            "namespace" => namespace = Some(content.parse()?),
            "component" => component = Some(content.parse()?),
            "stage" => stage = Some(content.parse()?),
            other => {
                return Err(syn::Error::new(
                    name.span(),
                    format!("unknown origin field `{other}`; expected `namespace`, `component`, or `stage`"),
                ));
            }
        }
        if content.peek(Token![,]) {
            content.parse::<Token![,]>()?;
        }
    }

    Ok(OriginSpec {
        namespace: namespace.ok_or_else(|| syn::Error::new(input.span(), "missing origin field `namespace`"))?,
        component: component.ok_or_else(|| syn::Error::new(input.span(), "missing origin field `component`"))?,
        stage,
    })
}

pub fn expand(input: DiagnosticInput) -> TokenStream {
    let code = &input.code;
    let severity = severity_tokens(&input.severity);
    let message = message_tokens(&input.message, code);
    let origin = origin_tokens(&input.origin);

    quote! {
        ::diagnostic::Diagnostic::new(
            ::diagnostic::DiagnosticCode::new(#code),
            #severity,
            #origin,
            #message,
        )
    }
}

fn severity_tokens(severity: &Severity) -> TokenStream {
    match severity {
        Severity::Bug => quote! { ::diagnostic::DiagnosticSeverity::Bug },
        Severity::Error => quote! { ::diagnostic::DiagnosticSeverity::Error },
        Severity::Warning => quote! { ::diagnostic::DiagnosticSeverity::Warning },
        Severity::Info => quote! { ::diagnostic::DiagnosticSeverity::Info },
        Severity::Hint => quote! { ::diagnostic::DiagnosticSeverity::Hint },
    }
}

fn message_tokens(message: &MessageSpec, code: &LitStr) -> TokenStream {
    match message {
        MessageSpec::Text(text) => quote! {
            ::diagnostic::Message::new(#code).with_fallback(#text)
        },
        MessageSpec::Structured { key, fallback } => match fallback {
            Some(fallback) => quote! {
                ::diagnostic::Message::new(#key).with_fallback(#fallback)
            },
            None => quote! {
                ::diagnostic::Message::new(#key)
            },
        },
    }
}

fn origin_tokens(origin: &Option<OriginSpec>) -> TokenStream {
    match origin {
        Some(origin) => {
            let namespace = &origin.namespace;
            let component = &origin.component;
            let stage = &origin.stage;
            match stage {
                Some(stage) => quote! {
                    ::diagnostic::DiagnosticOrigin::new(#namespace, #component).with_stage(#stage)
                },
                None => quote! {
                    ::diagnostic::DiagnosticOrigin::new(#namespace, #component)
                },
            }
        }
        None => quote! {
            ::diagnostic::DiagnosticOrigin::new("macro", "diagnostic-macro")
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn expands_structured_diagnostic() {
        let input: DiagnosticInput = parse_quote!(code: "macro.test.sample", severity: error, message: "sample message");
        let expanded = expand(input);
        let expected = quote! {
            ::diagnostic::Diagnostic::new(
                ::diagnostic::DiagnosticCode::new("macro.test.sample"),
                ::diagnostic::DiagnosticSeverity::Error,
                ::diagnostic::DiagnosticOrigin::new("macro", "diagnostic-macro"),
                ::diagnostic::Message::new("macro.test.sample").with_fallback("sample message"),
            )
        };
        assert_eq!(expanded.to_string(), expected.to_string());
    }

    #[test]
    fn rejects_unknown_field() {
        let source = parse_quote!(code: "macro.test.invalid", severity: error, message: "invalid", typo: "field");
        assert!(syn::parse2::<DiagnosticInput>(source).is_err());
    }
}
