# `boost-http`

Transport-independent HTTP request and response primitives for the rust-boost workspace.

The crate owns request and response data, headers, bounded response bodies, transport errors,
and the replaceable [`Transport`] boundary. The built-in `HttpTransport` owns synchronous
HTTP/1.1 framing and optionally uses `rustls` for HTTPS. It does not depend on a high-level HTTP
client or an asynchronous runtime.

`boost-http` does not own crawling, robots policy, response caching, format detection, or document
parsing. Those policies belong to the product that consumes the transport boundary. HTTP body
compression and character-set interpretation are intentionally outside this low-level layer.
