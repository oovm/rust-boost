#![doc = include_str!("readme.md")]
#![deny(missing_docs)]

use std::{
    collections::BTreeMap,
    error::Error as StdError,
    fmt::{Display, Formatter},
    io,
    str::FromStr,
};

pub use url::Url;

/// HTTP methods supported by [`Request`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Method {
    /// `GET`.
    #[default]
    Get,
    /// `HEAD`.
    Head,
    /// `POST`.
    Post,
    /// `PUT`.
    Put,
    /// `PATCH`.
    Patch,
    /// `DELETE`.
    Delete,
    /// `OPTIONS`.
    Options,
    /// `TRACE`.
    Trace,
    /// An extension method.
    Custom(String),
}

impl Method {
    /// Return the wire representation of the method.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Get => "GET",
            Self::Head => "HEAD",
            Self::Post => "POST",
            Self::Put => "PUT",
            Self::Patch => "PATCH",
            Self::Delete => "DELETE",
            Self::Options => "OPTIONS",
            Self::Trace => "TRACE",
            Self::Custom(value) => value,
        }
    }
}

impl FromStr for Method {
    type Err = RequestError;

    /// Parse a known method case-insensitively, or keep any other HTTP token as [`Method::Custom`].
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let method = match value.trim().to_ascii_uppercase().as_str() {
            "GET" => Self::Get,
            "HEAD" => Self::Head,
            "POST" => Self::Post,
            "PUT" => Self::Put,
            "PATCH" => Self::Patch,
            "DELETE" => Self::Delete,
            "OPTIONS" => Self::Options,
            "TRACE" => Self::Trace,
            _ if is_token(value.trim()) => Self::Custom(value.trim().to_string()),
            _ => return Err(RequestError::Unsupported(format!("invalid HTTP method `{value}`"))),
        };
        Ok(method)
    }
}

/// Case-insensitive HTTP headers that keep every repeated field value.
///
/// Names are stored in lower case and values keep the order in which they were appended.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Headers {
    entries: BTreeMap<String, Vec<String>>,
}

impl Headers {
    /// Create an empty header collection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Return whether no header is present.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Return the first value of a header.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.get_all(name).first().map(String::as_str)
    }

    /// Return every value of a header in append order.
    pub fn get_all(&self, name: &str) -> &[String] {
        self.entries.get(&name.to_ascii_lowercase()).map(Vec::as_slice).unwrap_or(&[])
    }

    /// Return whether a header has at least one value.
    pub fn contains(&self, name: &str) -> bool {
        self.entries.contains_key(&name.to_ascii_lowercase())
    }

    /// Replace every existing value of a header.
    pub fn set(&mut self, name: impl Into<String>, value: impl Into<String>) {
        let name = name.into().to_ascii_lowercase();
        self.entries.insert(name, vec![value.into()]);
    }

    /// Add a value after the values a header already has.
    pub fn append(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.entries.entry(name.into().to_ascii_lowercase()).or_default().push(value.into());
    }

    /// Remove a header together with all of its values.
    pub fn remove(&mut self, name: &str) {
        self.entries.remove(&name.to_ascii_lowercase());
    }

    /// Iterate over every name and value pair, repeating names that carry several values.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> + '_ {
        self.entries.iter().flat_map(|(name, values)| values.iter().map(move |value| (name.as_str(), value.as_str())))
    }

    /// Iterate over distinct header names.
    pub fn names(&self) -> impl Iterator<Item = &str> + '_ {
        self.entries.keys().map(String::as_str)
    }
}

/// A request body and its metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    /// HTTP method.
    pub method: Method,
    /// Target URL.
    pub url: Url,
    /// Request headers with case-insensitive names.
    pub headers: Headers,
    /// Raw request body.
    pub body: Vec<u8>,
}

impl Request {
    /// Create a request for a URL.
    pub fn new(method: Method, url: Url) -> Self {
        Self { method, url, headers: Headers::new(), body: Vec::new() }
    }

    /// Set a header, replacing every existing value with the same case-insensitive name.
    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.set_header(name, value);
        self
    }

    /// Set a header, replacing every existing value with the same case-insensitive name.
    pub fn set_header(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.headers.set(name, value);
    }

    /// Add another value to a repeated header.
    pub fn append_header(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.headers.append(name, value);
    }

    /// Set the raw request body.
    pub fn with_body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = body.into();
        self
    }
}

/// A response returned by a [`Transport`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    /// URL after transport-level redirects.
    pub url: Url,
    /// Numeric HTTP status code.
    pub status: u16,
    /// Response headers with case-insensitive names, including repeated fields.
    pub headers: Headers,
    /// Raw response body.
    pub body: Vec<u8>,
    /// Redirect responses followed before this response.
    pub redirects: Vec<Redirect>,
}

/// One redirect followed by a transport.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Redirect {
    /// URL that returned the redirect response.
    pub url: Url,
    /// Numeric redirect status code.
    pub status: u16,
}

impl Response {
    /// Return whether the status code is in the successful `2xx` range.
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }

    /// Return whether the status code is in the redirect `3xx` range.
    pub fn is_redirection(&self) -> bool {
        (300..400).contains(&self.status)
    }

    /// Return whether the status code is in the client or server error range.
    pub fn is_error(&self) -> bool {
        self.status >= 400
    }
}

/// Errors produced while constructing or sending a request.
#[derive(Debug)]
pub enum RequestError {
    /// The selected transport cannot handle the request.
    Unsupported(String),
    /// The transport failed before a response was produced.
    Transport(String),
    /// Reading a response body failed.
    Body(io::Error),
    /// The response body exceeded the configured limit.
    BodyTooLarge {
        /// Maximum permitted body size.
        limit: usize,
    },
    /// The response headers exceeded the configured limit.
    HeaderTooLarge {
        /// Maximum permitted header block size.
        limit: usize,
    },
}

impl Display for RequestError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported(message) => write!(formatter, "unsupported request: {message}"),
            Self::Transport(message) => write!(formatter, "transport error: {message}"),
            Self::Body(error) => write!(formatter, "response body error: {error}"),
            Self::BodyTooLarge { limit } => write!(formatter, "response body exceeds {limit} bytes"),
            Self::HeaderTooLarge { limit } => write!(formatter, "response headers exceed {limit} bytes"),
        }
    }
}

impl StdError for RequestError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Body(error) => Some(error),
            _ => None,
        }
    }
}

/// A synchronous request executor.
pub trait Transport {
    /// Send one request and return its response.
    fn send(&self, request: &Request) -> Result<Response, RequestError>;
}

/// Send one request through a transport.
pub fn send<T: Transport + ?Sized>(transport: &T, request: &Request) -> Result<Response, RequestError> {
    transport.send(request)
}

/// Return whether a value is a valid HTTP token, as used by methods and header names.
pub(crate) fn is_token(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&byte))
}

mod http_transport;

pub use http_transport::{HttpTransport, HttpTransportConfig};
