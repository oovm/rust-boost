# `console`

Workspace-wide structured logging and event facade.

`console` owns `ConsoleEvent`, span context, filters, sinks, and generic terminal styling helpers. It does not depend on `diagnostic`. Domain crates can emit ordinary events through `console::event!` without importing parser or diagnostic models.

`diagnostic` may project structured diagnostics into `ConsoleEvent` with `EventKind::Diagnostic`, preserving code, severity, message key, arguments, locations, actions, and cause.
