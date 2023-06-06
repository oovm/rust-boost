//! Request building, transport dispatch, and status helpers.

use std::str::FromStr;

use boost_http::{Headers, Method, Request, RequestError, Response, Transport, Url, send};

/// Transport that echoes the request back, so the public `send` entry has a real implementation.
struct EchoTransport;

impl Transport for EchoTransport {
    fn send(&self, request: &Request) -> Result<Response, RequestError> {
        Ok(Response { url: request.url.clone(), status: 201, headers: Headers::new(), body: request.body.clone(), redirects: Vec::new() })
    }
}

/// Builds a response that only carries the status under test.
fn with_status(status: u16) -> Response {
    Response { url: Url::parse("https://example.test").unwrap(), status, headers: Headers::new(), body: Vec::new(), redirects: Vec::new() }
}

#[test]
fn builder_normalizes_header_names_and_preserves_body() {
    let request = Request::new(Method::Post, Url::parse("https://example.test/items").unwrap())
        .with_header("Content-Type", "application/octet-stream")
        .with_body(vec![1, 2, 3]);

    assert_eq!(request.headers.get("CONTENT-TYPE"), Some("application/octet-stream"));
    let response = send(&EchoTransport, &request).unwrap();
    assert_eq!(response.status, 201);
    assert!(response.is_success());
    assert_eq!(response.body, vec![1, 2, 3]);
}

#[test]
fn repeated_headers_keep_every_value_in_append_order() {
    let mut request = Request::new(Method::Get, Url::parse("https://example.test").unwrap());
    request.set_header("Set-Cookie", "a=1");
    request.append_header("set-cookie", "b=2");

    assert_eq!(request.headers.get("set-cookie"), Some("a=1"));
    assert_eq!(request.headers.get_all("Set-Cookie").iter().map(String::as_str).collect::<Vec<_>>(), ["a=1", "b=2"]);
    assert_eq!(request.headers.iter().collect::<Vec<_>>(), [("set-cookie", "a=1"), ("set-cookie", "b=2")]);
    request.set_header("set-cookie", "c=3");
    assert_eq!(request.headers.get_all("set-cookie").iter().map(String::as_str).collect::<Vec<_>>(), ["c=3"]);
}

#[test]
fn method_parsing_keeps_known_names_and_valid_custom_tokens() {
    assert_eq!(Method::from_str("get").unwrap(), Method::Get);
    assert_eq!(Method::from_str("DELETE").unwrap(), Method::Delete);
    assert_eq!(Method::from_str("PURGE").unwrap(), Method::Custom("PURGE".to_string()));
    assert!(matches!(Method::from_str("bad method"), Err(RequestError::Unsupported(_))));
}

#[test]
fn status_helpers_partition_http_status_ranges() {
    for status in [200, 204, 299] {
        assert!(with_status(status).is_success());
    }
    for status in [301, 399] {
        assert!(with_status(status).is_redirection());
    }
    assert!(with_status(500).is_error());
}
