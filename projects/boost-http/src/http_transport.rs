use std::{
    io::{self, BufReader, Read, Write},
    net::{TcpStream, ToSocketAddrs},
    time::{Duration, Instant},
};

#[cfg(feature = "tls")]
use rustls::pki_types::ServerName;
#[cfg(feature = "tls")]
use rustls::{ClientConfig, ClientConnection, RootCertStore, StreamOwned};
#[cfg(feature = "tls")]
use std::sync::Arc;

use super::{Headers, Method, Redirect, Request, RequestError, Response, Transport, is_token};

/// Configuration for the built-in HTTP/1.1 transport.
#[derive(Debug, Clone)]
pub struct HttpTransportConfig {
    /// Maximum response body size after transfer decoding.
    pub max_body_bytes: usize,
    /// Maximum number of redirects to follow.
    pub max_redirects: u32,
    /// Whole request-chain timeout.
    pub timeout: Duration,
    /// Maximum response header block size.
    pub max_header_bytes: usize,
}

impl Default for HttpTransportConfig {
    fn default() -> Self {
        Self { max_body_bytes: 8 * 1024 * 1024, max_redirects: 8, timeout: Duration::from_secs(30), max_header_bytes: 64 * 1024 }
    }
}

/// Minimal synchronous HTTP/1.1 transport owned by `request`.
#[derive(Debug, Clone, Default)]
pub struct HttpTransport {
    /// Transport limits and timeout policy.
    pub config: HttpTransportConfig,
}

impl HttpTransport {
    /// Create a transport with conservative defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a transport with explicit limits.
    pub fn with_config(config: HttpTransportConfig) -> Self {
        Self { config }
    }
}

enum Connection {
    Tcp(DeadlineSocket),
    #[cfg(feature = "tls")]
    Tls(Box<StreamOwned<ClientConnection, DeadlineSocket>>),
}

struct DeadlineSocket {
    stream: TcpStream,
    deadline: Instant,
}

impl DeadlineSocket {
    fn remaining(&self) -> io::Result<Duration> {
        self.deadline
            .checked_duration_since(Instant::now())
            .filter(|duration| !duration.is_zero())
            .ok_or_else(|| io::Error::new(io::ErrorKind::TimedOut, "request deadline exceeded"))
    }
}

impl Read for DeadlineSocket {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        self.stream.set_read_timeout(Some(self.remaining()?))?;
        self.stream.read(buffer)
    }
}

impl Write for DeadlineSocket {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.stream.set_write_timeout(Some(self.remaining()?))?;
        self.stream.write(buffer)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.remaining()?;
        self.stream.flush()
    }
}

impl Read for Connection {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Tcp(stream) => stream.read(bytes),
            #[cfg(feature = "tls")]
            Self::Tls(stream) => stream.read(bytes),
        }
    }
}

impl Write for Connection {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        match self {
            Self::Tcp(stream) => stream.write(bytes),
            #[cfg(feature = "tls")]
            Self::Tls(stream) => stream.write(bytes),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Self::Tcp(stream) => stream.flush(),
            #[cfg(feature = "tls")]
            Self::Tls(stream) => stream.flush(),
        }
    }
}

impl Transport for HttpTransport {
    fn send(&self, request: &Request) -> Result<Response, RequestError> {
        validate_url(&request.url)?;
        let started = Instant::now();
        let mut current_url = request.url.clone();
        current_url.set_fragment(None);
        let mut method = request.method.clone();
        let mut headers = Headers::new();
        if !is_token(request.method.as_str()) {
            return Err(RequestError::Unsupported("invalid HTTP method".into()));
        }
        for (name, value) in request.headers.iter() {
            validate_header(name, value)?;
            headers.append(name, value);
        }
        let mut body = request.body.clone();
        let mut redirects = Vec::new();

        for _ in 0..=self.config.max_redirects {
            let remaining = remaining_timeout(self.config.timeout, started.elapsed())?;
            let mut response = self.send_once(&current_url, &method, &headers, &body, remaining)?;
            if self.config.max_redirects > 0 && is_redirect(response.status) {
                if redirects.len() >= self.config.max_redirects as usize {
                    return Err(RequestError::Transport(format!("redirect chain exceeded {} hops", self.config.max_redirects)));
                }
                let location =
                    response.headers.get("location").ok_or_else(|| RequestError::Transport("redirect without location header".into()))?;
                let mut next_url = current_url.join(location).map_err(|error| RequestError::Transport(error.to_string()))?;
                next_url.set_fragment(None);
                validate_redirect(&current_url, &next_url)?;
                if current_url.origin() != next_url.origin() {
                    remove_sensitive_headers(&mut headers);
                }
                if should_switch_to_get(response.status, &method) {
                    method = Method::Get;
                    body.clear();
                    remove_entity_headers(&mut headers);
                }
                redirects.push(Redirect { url: current_url, status: response.status });
                current_url = next_url;
                continue;
            }
            response.url = current_url;
            response.redirects = redirects;
            return Ok(response);
        }
        Err(RequestError::Transport("redirect loop exhausted".into()))
    }
}

impl HttpTransport {
    fn send_once(&self, url: &url::Url, method: &Method, headers: &Headers, body: &[u8], timeout: Duration) -> Result<Response, RequestError> {
        let host = url.host_str().ok_or_else(|| RequestError::Unsupported("URL has no host".into()))?;
        let network_host = host.trim_start_matches('[').trim_end_matches(']');
        let deadline = Instant::now().checked_add(timeout).ok_or_else(|| RequestError::Unsupported("timeout is too large".into()))?;
        let port = url.port_or_known_default().ok_or_else(|| RequestError::Unsupported("URL has no port".into()))?;
        let addresses = (network_host, port).to_socket_addrs().map_err(|error| RequestError::Transport(error.to_string()))?.collect::<Vec<_>>();
        let mut connected = None;
        let mut last_error = "host resolved to no addresses".to_string();
        for address in addresses {
            let remaining = deadline
                .checked_duration_since(Instant::now())
                .filter(|duration| !duration.is_zero())
                .ok_or_else(|| RequestError::Transport("request deadline exceeded".into()))?;
            match TcpStream::connect_timeout(&address, remaining) {
                Ok(stream) => {
                    connected = Some(stream);
                    break;
                }
                Err(error) => last_error = error.to_string(),
            }
        }
        let stream = DeadlineSocket { stream: connected.ok_or(RequestError::Transport(last_error))?, deadline };
        let mut connection = secure_connection(url.scheme(), network_host, stream)?;
        let path = match url.query() {
            Some(query) => format!("{}?{query}", if url.path().is_empty() { "/" } else { url.path() }),
            None if url.path().is_empty() => "/".to_string(),
            None => url.path().to_string(),
        };
        let host_header = match url.port() {
            Some(port) => format!("{host}:{port}"),
            None => host.to_string(),
        };
        let mut wire = format!("{} {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n", method.as_str(), path, host_header);
        if headers.get_all("content-length").len() > 1 {
            return Err(RequestError::Unsupported("request has more than one Content-Length header".into()));
        }
        for (name, value) in headers.iter() {
            validate_header(name, value)?;
            if matches!(name, "transfer-encoding" | "expect" | "upgrade") {
                return Err(RequestError::Unsupported("request streaming, expectation and upgrade headers are not supported".into()));
            }
            if name == "host" || name == "connection" {
                continue;
            }
            if name == "content-length" && value.parse::<usize>().ok() != Some(body.len()) {
                return Err(RequestError::Unsupported("request Content-Length does not match body".into()));
            }
            wire.push_str(name);
            wire.push_str(": ");
            wire.push_str(value);
            wire.push_str("\r\n");
        }
        if !headers.contains("content-length") {
            wire.push_str(&format!("Content-Length: {}\r\n", body.len()));
        }
        wire.push_str("\r\n");
        connection.write_all(wire.as_bytes()).map_err(RequestError::Body)?;
        connection.write_all(body).map_err(RequestError::Body)?;
        connection.flush().map_err(RequestError::Body)?;
        read_response(&mut BufReader::new(connection), url.clone(), method, self.config.max_header_bytes, self.config.max_body_bytes)
    }
}

fn read_response<R: Read>(
    connection: &mut R,
    url: url::Url,
    method: &Method,
    max_header_bytes: usize,
    max_body_bytes: usize,
) -> Result<Response, RequestError> {
    let mut header_budget = max_header_bytes;
    let (status, headers) = loop {
        let mut head = Vec::new();
        let mut byte = [0u8; 1];
        while !head.ends_with(b"\r\n\r\n") {
            if head.len() >= header_budget {
                return Err(RequestError::HeaderTooLarge { limit: max_header_bytes });
            }
            connection.read_exact(&mut byte).map_err(RequestError::Body)?;
            head.push(byte[0]);
        }
        let text = std::str::from_utf8(&head).map_err(|error| RequestError::Transport(format!("response headers are not UTF-8: {error}")))?;
        let mut lines = text.split("\r\n");
        let status_line = lines.next().unwrap_or("");
        let mut parts = status_line.splitn(3, ' ');
        let version = parts.next().unwrap_or("");
        let code = parts.next().unwrap_or("");
        if !matches!(version, "HTTP/1.0" | "HTTP/1.1") || code.len() != 3 || !code.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(RequestError::Transport("invalid HTTP status line".into()));
        }
        let status = code.parse::<u16>().map_err(|_| RequestError::Transport("invalid HTTP status code".into()))?;
        if status < 100 || status == 101 {
            return Err(RequestError::Unsupported("invalid status or protocol upgrade".into()));
        }
        let mut headers = Headers::new();
        for line in lines {
            if line.is_empty() {
                break;
            }
            let (name, value) = line.split_once(':').ok_or_else(|| RequestError::Transport("invalid response header".into()))?;
            validate_header(name, value)?;
            headers.append(name, value.trim().to_string());
        }
        header_budget -= head.len();
        if status >= 200 {
            break (status, headers);
        }
    };
    let lengths = headers.get_all("content-length");
    if lengths.len() > 1 {
        return Err(RequestError::Transport("response has more than one Content-Length header".into()));
    }
    let length = lengths
        .first()
        .map(|value| {
            if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(RequestError::Transport("invalid Content-Length".into()));
            }
            value.parse::<usize>().map_err(|_| RequestError::Transport("Content-Length overflows".into()))
        })
        .transpose()?;
    let transfer_encodings = headers.get_all("transfer-encoding");
    if !transfer_encodings.is_empty() && length.is_some() {
        return Err(RequestError::Transport("conflicting Transfer-Encoding and Content-Length".into()));
    }
    if transfer_encodings.len() > 1 || transfer_encodings.first().is_some_and(|value| !value.eq_ignore_ascii_case("chunked")) {
        return Err(RequestError::Unsupported("unsupported Transfer-Encoding".into()));
    }
    let chunked = !transfer_encodings.is_empty();
    let body = if *method == Method::Head || matches!(status, 204 | 304) {
        Vec::new()
    }
    else if chunked {
        read_chunked_body(connection, max_body_bytes, max_header_bytes)?
    }
    else if let Some(length) = length {
        if length > max_body_bytes {
            return Err(RequestError::BodyTooLarge { limit: max_body_bytes });
        }
        let mut body = vec![0u8; length];
        connection.read_exact(&mut body).map_err(RequestError::Body)?;
        body
    }
    else {
        read_until_close(connection, max_body_bytes)?
    };
    Ok(Response { url, status, headers, body, redirects: Vec::new() })
}

fn read_chunked_body<R: Read>(connection: &mut R, limit: usize, header_limit: usize) -> Result<Vec<u8>, RequestError> {
    let mut output = Vec::new();
    let mut metadata_remaining = header_limit;
    loop {
        let line = read_line(connection, metadata_remaining.min(4096))?;
        metadata_remaining = metadata_remaining.saturating_sub(line.len() + 2);
        let size_text = line.split(';').next().unwrap_or("");
        if size_text.is_empty() || !size_text.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(RequestError::Transport("invalid chunk size".into()));
        }
        let size = usize::from_str_radix(size_text, 16).map_err(|_| RequestError::Transport("invalid chunk size".into()))?;
        if size == 0 {
            loop {
                let trailer = read_line(connection, metadata_remaining)?;
                metadata_remaining = metadata_remaining.saturating_sub(trailer.len() + 2);
                if trailer.is_empty() {
                    break;
                }
                let (name, value) = trailer.split_once(':').ok_or_else(|| RequestError::Transport("invalid trailer".into()))?;
                validate_header(name, value)?;
                if matches!(name.to_ascii_lowercase().as_str(), "content-length" | "transfer-encoding" | "host") {
                    return Err(RequestError::Transport("invalid framing trailer".into()));
                }
            }
            break;
        }
        if output.len().saturating_add(size) > limit {
            return Err(RequestError::BodyTooLarge { limit });
        }
        let start = output.len();
        output.resize(start + size, 0);
        connection.read_exact(&mut output[start..]).map_err(RequestError::Body)?;
        let mut crlf = [0u8; 2];
        connection.read_exact(&mut crlf).map_err(RequestError::Body)?;
        if crlf != *b"\r\n" {
            return Err(RequestError::Transport("invalid chunk terminator".into()));
        }
    }
    Ok(output)
}

fn read_until_close<R: Read>(connection: &mut R, limit: usize) -> Result<Vec<u8>, RequestError> {
    let mut body = Vec::new();
    let mut buffer = [0u8; 8192];
    loop {
        let read = connection.read(&mut buffer).map_err(RequestError::Body)?;
        if read == 0 {
            break;
        }
        if body.len().saturating_add(read) > limit {
            return Err(RequestError::BodyTooLarge { limit });
        }
        body.extend_from_slice(&buffer[..read]);
    }
    Ok(body)
}

fn read_line<R: Read>(connection: &mut R, limit: usize) -> Result<String, RequestError> {
    let mut line = Vec::new();
    let mut byte = [0u8; 1];
    while !line.ends_with(b"\r\n") {
        if line.len() >= limit {
            return Err(RequestError::HeaderTooLarge { limit });
        }
        connection.read_exact(&mut byte).map_err(RequestError::Body)?;
        line.push(byte[0]);
    }
    String::from_utf8(line[..line.len() - 2].to_vec()).map_err(|error| RequestError::Transport(error.to_string()))
}

fn secure_connection(scheme: &str, host: &str, stream: DeadlineSocket) -> Result<Connection, RequestError> {
    match scheme {
        "http" => Ok(Connection::Tcp(stream)),
        "https" => {
            #[cfg(feature = "tls")]
            {
                let mut roots = RootCertStore::empty();
                roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
                let config = ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
                    .with_safe_default_protocol_versions()
                    .map_err(|error| RequestError::Transport(error.to_string()))?
                    .with_root_certificates(roots)
                    .with_no_client_auth();
                let name = ServerName::try_from(host.to_string()).map_err(|error| RequestError::Unsupported(error.to_string()))?;
                let connection = ClientConnection::new(Arc::new(config), name).map_err(|error| RequestError::Transport(error.to_string()))?;
                Ok(Connection::Tls(Box::new(StreamOwned::new(connection, stream))))
            }
            #[cfg(not(feature = "tls"))]
            {
                let _ = (host, stream);
                Err(RequestError::Unsupported("HTTPS requires the `tls` feature".into()))
            }
        }
        _ => Err(RequestError::Unsupported(format!("unsupported URL scheme `{scheme}`"))),
    }
}

fn validate_url(url: &url::Url) -> Result<(), RequestError> {
    if url.host_str().is_none() || url.username() != "" || url.password().is_some() {
        return Err(RequestError::Unsupported("URL must have a host and no credentials".into()));
    }
    if !matches!(url.scheme(), "http" | "https") {
        return Err(RequestError::Unsupported(format!("unsupported URL scheme `{}`", url.scheme())));
    }
    #[cfg(not(feature = "tls"))]
    if url.scheme() == "https" {
        return Err(RequestError::Unsupported("HTTPS requires the `tls` feature".into()));
    }
    Ok(())
}

fn validate_redirect(from: &url::Url, to: &url::Url) -> Result<(), RequestError> {
    validate_url(to)?;
    if from.scheme() == "https" && to.scheme() == "http" {
        return Err(RequestError::Unsupported("HTTPS downgrade redirect is not permitted".into()));
    }
    Ok(())
}

fn validate_header(name: &str, value: &str) -> Result<(), RequestError> {
    if !is_token(name) || value.bytes().any(|byte| byte < 0x20 && byte != b'\t' || byte == 0x7f) {
        return Err(RequestError::Unsupported("invalid header name or value".into()));
    }
    Ok(())
}

fn remaining_timeout(total: Duration, elapsed: Duration) -> Result<Duration, RequestError> {
    total.checked_sub(elapsed).filter(|value| !value.is_zero()).ok_or_else(|| RequestError::Transport("whole-request timeout exceeded".into()))
}

fn is_redirect(status: u16) -> bool {
    matches!(status, 301 | 302 | 303 | 307 | 308)
}
fn should_switch_to_get(status: u16, method: &Method) -> bool {
    (status == 303 && *method != Method::Head) || (matches!(status, 301 | 302) && *method == Method::Post)
}
fn remove_sensitive_headers(headers: &mut Headers) {
    for name in ["authorization", "proxy-authorization", "cookie", "host"] {
        headers.remove(name);
    }
}
fn remove_entity_headers(headers: &mut Headers) {
    for name in ["content-length", "content-type", "transfer-encoding"] {
        headers.remove(name);
    }
}
