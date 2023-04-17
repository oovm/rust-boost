#![doc = include_str!("readme.md")]
#![deny(missing_docs)]

use std::{
    collections::BTreeMap,
    error::Error as StdError,
    fmt::{Display, Formatter},
    io,
};

use url::Url;

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

/// A request body and its metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    /// HTTP method.
    pub method: Method,
    /// Target URL.
    pub url: Url,
    /// Case-insensitive request headers stored using lower-case keys.
    pub headers: BTreeMap<String, String>,
    /// Raw request body.
    pub body: Vec<u8>,
}

impl Request {
    /// Create a request for a URL.
    pub fn new(method: Method, url: Url) -> Self {
        Self { method, url, headers: BTreeMap::new(), body: Vec::new() }
    }

    /// Set a header, replacing an existing value with the same case-insensitive name.
    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.set_header(name, value);
        self
    }

    /// Set a header, replacing an existing value with the same case-insensitive name.
    pub fn set_header(&mut self, name: impl Into<String>, value: impl Into<String>) {
        self.headers.insert(name.into().to_ascii_lowercase(), value.into());
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
    /// Case-insensitive response headers stored using lower-case keys.
    pub headers: BTreeMap<String, String>,
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

mod http_transport;

pub use http_transport::{HttpTransport, HttpTransportConfig};
