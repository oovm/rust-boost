//! Response parsing, redirect handling, and budget limits of the HTTP transport.

use std::{
    io::{Read, Write},
    net::TcpListener,
    thread,
};

use boost_http::{HttpTransport, HttpTransportConfig, Method, Request, RequestError, Transport};
use url::Url;

/// Serves exactly one response over a local TCP connection and returns its URL.
fn serve(response: &'static str) -> Url {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut request = Vec::new();
        let mut byte = [0u8; 1];
        while !request.ends_with(b"\r\n\r\n") {
            stream.read_exact(&mut byte).unwrap();
            request.push(byte[0]);
        }
        stream.write_all(response.as_bytes()).unwrap();
    });
    Url::parse(&format!("http://{address}/resource?part=1")).unwrap()
}

#[test]
fn parses_content_length_and_non_success_status() {
    let url = serve("HTTP/1.1 418 I'm a teapot\r\nContent-Length: 3\r\n\r\ntea");
    let response = HttpTransport::new().send(&Request::new(Method::Get, url)).unwrap();
    assert_eq!(response.status, 418);
    assert_eq!(response.body, b"tea");
    assert!(response.is_error());
}

#[test]
fn decodes_chunked_body() {
    let url = serve("HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n5\r\nhello\r\n0\r\n\r\n");
    let response = HttpTransport::new().send(&Request::new(Method::Get, url)).unwrap();
    assert_eq!(response.body, b"hello");
}

#[test]
fn keeps_repeated_response_headers() {
    let url = serve("HTTP/1.1 200 OK\r\nSet-Cookie: a=1\r\nSet-Cookie: b=2\r\nContent-Length: 0\r\n\r\n");
    let response = HttpTransport::new().send(&Request::new(Method::Get, url)).unwrap();
    assert_eq!(response.headers.get_all("set-cookie").iter().map(String::as_str).collect::<Vec<_>>(), ["a=1", "b=2"]);
}

#[test]
#[ignore = "requires outbound HTTPS access"]
fn fetches_https_resource_with_webpki_roots() {
    let url = Url::parse("https://example.com/").unwrap();
    let response = HttpTransport::new().send(&Request::new(Method::Get, url)).unwrap();
    assert_eq!(response.status, 200);
    assert!(!response.body.is_empty());
}

#[test]
fn rejects_response_header_budget() {
    let url = serve("HTTP/1.1 200 OK\r\nX-Large: value\r\nContent-Length: 0\r\n\r\n");
    let transport = HttpTransport::with_config(HttpTransportConfig { max_header_bytes: 10, ..HttpTransportConfig::default() });
    assert!(matches!(transport.send(&Request::new(Method::Get, url)), Err(RequestError::HeaderTooLarge { limit: 10 })));
}
