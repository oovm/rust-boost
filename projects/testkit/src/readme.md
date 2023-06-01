# `testkit`

Structured test identity, execution results, assertions, and text reports for the rust-boost family.

`testkit` keeps the facts of one test execution instead of reducing it to a pass/fail bit:

- A `TestDescriptor` records the stable name, module, source location, expectation, and advertised capabilities of an entry.
- A `TestContext` is created per case and carries the descriptor and the skip state, so a body never has to reach global mutable state.
- A `TestResult` records the outcome, the ordered failures, the skip reason, and the observed duration.
- A `TestFailure` keeps its phase, kind, message, source location, and structured operands. The text projection in `testkit::report` only renders those facts and never reinterprets them.

Assertions in `testkit::assert` are ordinary Rust macros that fail with a structured `TestFailure` payload carried by a panic. `testkit::execute` catches that payload, so `assert!`-style control flow still stops the body immediately, while the result keeps the location and operands that the default panic hook would flatten into text. `testkit::run` is the standard-harness entry: it executes the body and panics with the rendered report when the result is a failure, which keeps a generated wrapper usable from `cargo test`.

The default `macros` feature re-exports `#[testkit::test]` from `testkit-macro`. The attribute turns a free function into a standard `#[test]` wrapper, builds the descriptor from `module_path!()` and the invocation location, and accepts a body with no arguments or one `&mut TestContext` and a return type of `()` or `Result<(), E>`. `name = "..."` overrides the descriptor name, and `expected = "failure"` (or `expected_failure`) declares an expected failure. Invalid signatures and unsupported attributes are rejected at compile time instead of becoming runtime errors.

The current slice covers discovery identity, synchronous execution, structured assertions, skip and expected-failure semantics, text reports, and the `#[testkit::test]` wrapper. Fixtures, parameterized cases, the benchmark engine, coverage probes, traces, serialization, asynchronous execution, and a committed compile-fail regression harness remain planned capabilities and are not claimed by this crate yet. In particular, this crate does not replace `#[test]`, `cargo test`, or compiler-level coverage, and it does not report a timeout it cannot enforce.
