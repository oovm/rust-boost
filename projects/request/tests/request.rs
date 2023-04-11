//! Request building, transport dispatch, and status helpers.

use std::collections::BTreeMap;

use request::{Method, Request, RequestError, Response, Transport, send};
use url::Url;

/// Transport that echoes the request back, so the public `send` entry has a real implementation.
struct EchoTransport;

impl Transport for EchoTransport {
    fn send(&self, request: &Request) -> Result<Response, RequestError> {
        Ok(Response {
            url: request.url.clone(),
            status: 201,
            headers: BTreeMap::new(),
            body: request.body.clone(),
            redirects: Vec::new(),
        })
    }
}

/// Builds a response that only carries the status under test.
fn with_status(status: u16) -> Response {
    Response {
        url: Url::parse("https://example.test").unwrap(),
        status,
        headers: BTreeMap::new(),
        body: Vec::new(),
        redirects: Vec::new(),
    }
}

#[test]
fn builder_normalizes_header_names_and_preserves_body() {
    let request = Request::new(Method::Post, Url::parse("https://example.test/items").unwrap())
        .with_header("Content-Type", "application/octet-stream")
        .with_body(vec![1, 2, 3]);

    assert_eq!(request.headers["content-type"], "application/octet-stream");
    let response = send(&EchoTransport, &request).unwrap();
    assert_eq!(response.status, 201);
    assert!(response.is_success());
    assert_eq!(response.body, vec![1, 2, 3]);
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
