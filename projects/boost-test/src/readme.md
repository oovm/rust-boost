# `boost-test`

Structured test identity, execution results, assertions, and text reports for the rust-boost family.

`boost-test` keeps the facts of one test execution instead of reducing it to a pass/fail bit:

- A `TestDescriptor` records the stable name, module, source location, expectation, and advertised capabilities of an entry.
- A `TestContext` is created per case and carries the descriptor and the skip state, so a body never has to reach global mutable state.
- A `TestResult` records the outcome, the ordered failures, the skip reason, and the observed duration.
- A `TestFailure` keeps its phase, kind, message, source location, and structured operands. The text projection in `boost_test::report` only renders those facts and never reinterprets them.

Assertions in `boost_test::assert` are ordinary Rust macros that fail with a structured `TestFailure` payload carried by a panic. `boost_test::execute` catches that payload, so `assert!`-style control flow still stops the body immediately, while the result keeps the location and operands that the default panic hook would flatten into text. `boost_test::run` is the standard-harness entry: it executes the body and panics with the rendered report when the result is a failure, which keeps a generated wrapper usable from `cargo test`.

The default `macros` feature re-exports `#[boost_test::test]`, `#[boost_test::fixture]`, and `#[boost_test::case]`. The test attribute turns a free function into a standard `#[test]` wrapper, builds the descriptor from `module_path!()` and the invocation location, and accepts a body with no arguments or one `&mut TestContext` and a return type of `()` or `Result<(), E>`. `name = "..."` overrides the descriptor name, `case = "..."` attaches a stable case identity, `expected = "failure"` (or `expected_failure`) declares an expected failure, and `fixture = path` runs the body against a typed fixture. Invalid signatures and unsupported attributes are rejected at compile time instead of becoming runtime errors.

A fixture wraps a setup function that takes no arguments or one `&mut TestContext` and returns either the fixture value or `Result<Value, E>`. The setup function stays callable, while a marker type with the same name lets `fixture = path` name it in type position. An optional `teardown = path` receives the value and may return `()` or `Result<(), E>`. The runner records setup failure in the setup phase and does not enter the body, releases the fixture after a passing, failing, panicking, or skipped body, and appends teardown failures after the original body failure instead of replacing it. A failure outside the body keeps the result failed even when the descriptor declares an expected failure.

A case table expands into one standard `#[test]` entry per stable case name. Entries are written as `"case name" => arguments`; a body with several parameters receives a tuple with exactly that many elements, while a body with one parameter receives the expression itself. The case name is recorded through `CaseId` and rendered as `module::name[case]`, so two runs of the same body stay distinguishable without an address, a thread id, or an execution order.

The current slice covers discovery identity, synchronous execution, structured assertions, skip and expected-failure semantics, text reports, the `#[boost_test::test]` wrapper, typed fixtures with setup/teardown phases, and parameterized cases. Per-case expectations and retry budgets, deterministic seeds, the benchmark engine, coverage probes, trace collection, serialization, asynchronous execution, and a committed compile-fail regression harness remain planned capabilities and are not claimed by this crate yet. In particular, this crate does not replace `#[test]`, `cargo test`, or compiler-level coverage, and it does not report a timeout it cannot enforce.
