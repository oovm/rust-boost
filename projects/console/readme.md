# `console`

Human-visible console subscriber and generic terminal styling.

`console` re-exports the shared `logger` facade under transitional `Console*` names and adds color, paint, and layout helpers. It depends on `logger` and `terminal`, not on `diagnostic`.

Domain crates should prefer `logger` for new code. `diagnostic` may still project structured diagnostics into `ConsoleEvent` / `LogEvent` with `EventKind::Diagnostic`.
