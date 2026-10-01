# diagnostic

Structured diagnostic facts for the rust-boost workspace.

## Modules

```text
model     one diagnostic fact
collect   DiagnosticSet and DiagnosticSink
wire      versioned serialization
source    opaque SourceRef contracts
render    renderer-neutral projection helpers
terminal  optional text rendering via `console` and `terminal`
```

## Core

- `Diagnostic`, `DiagnosticCode`, `DiagnosticSeverity`, `DiagnosticOrigin`
- `DiagnosticLocation`, `DiagnosticLabel`, `Message`, `DiagnosticAction`
- `DiagnosticSet`, `DiagnosticSink`

Enable the `serde` feature for JSON wire encoding via `diagnostic::wire`.

## Boundaries

The core crate does not provide business result wrappers, quick error facades,
source storage, or terminal policy. Enable the `terminal` feature to render text
diagnostics through `console` and `terminal`. Oak, Acorn, and Panduck keep their
own result types and only share `DiagnosticSet` as the common diagnostic container.
