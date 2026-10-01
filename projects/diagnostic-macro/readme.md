# `diagnostic-macro`

Procedural macros for building structured [`diagnostic`] records.

## `diagnose!`

Use `diagnose!` instead of abbreviated names such as `diag!`. The macro expands to a
structured `Diagnostic` value:

```rust
use diagnostic_macro::diagnose;

let record = diagnose! {
    code: "oak.syntax.unexpected-token",
    severity: error,
    message: "unexpected token",
    origin: {
        namespace: "oak",
        component: "xml",
        stage: "parse",
    },
};
```

Structured message keys are also supported:

```rust
let record = diagnose! {
    code: "oak.syntax.unexpected-token",
    severity: warning,
    message: {
        key: "oak.syntax.unexpected-token",
        fallback: "unexpected token",
    },
};
```

[`diagnostic`]: https://docs.rs/diagnostic
