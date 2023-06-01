# `testkit-macro`

Procedural macros for `testkit`.

`#[testkit::test]` turns a free function into a standard `#[test]` entry that builds a
`TestDescriptor` and calls `testkit::run`. The generated wrapper is ordinary Rust: the original body
keeps its name-derived source location, its `?` and `return` behaviour, and its result type.

Supported signatures:

- `fn name()`.
- `fn name(context: &mut testkit::TestContext)`.
- Either form with a `Result<(), E>` return type, where `E: Display + 'static`.

Supported attributes:

- `name = "..."` overrides the descriptor name without renaming the function.
- `expected = "failure"` declares an expected failure. `expected = "pass"` is the default.
- `expected_failure` is a shorthand for `expected = "failure"`.

The macro rejects `async`, generic, `const`, `unsafe`, `extern`, receiver-based, and non-unit result
signatures with a compile-time error at the offending span. Unsupported attributes such as
`timeout`, `fixture`, `case`, and `trace` are rejected rather than silently ignored until their
runtime contracts exist.
