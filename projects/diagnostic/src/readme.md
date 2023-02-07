# diagnostic

Structured diagnostic data model for the rust-boost workspace.

## Core

Crate root exports the unified contract:

- `Diagnostic`, `DiagnosticCode`, `DiagnosticSeverity`, `DiagnosticOrigin`
- `DiagnosticLocation`, `DiagnosticLabel`, `Message`
- `DiagnosticSet`, `DiagnosticEmitter`, `Report`

Enable the `serde` feature for JSON wire encoding via `diagnostic::json`.

## Terminal

Enable the `terminal` feature for legacy text rendering (`diagnostic::terminal`).

Terminal rendering consumes `source-cache` as an external provider and does not own source data.
