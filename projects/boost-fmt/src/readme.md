# `boost-fmt`

Human-visible console subscriber and generic terminal styling.

`boost-fmt` depends on `boost-log` and `boost-terminal`, not on `boost-diagnostic`. It provides color, paint, layout helpers, `render` for styled `LogEvent` output, and `install_global_subscriber` to attach the default stderr subscriber to the global `boost-log` facade.

Domain crates should use `boost-log` for structured events, spans, and sinks. Use `boost-fmt` only for terminal styling or when installing the default human-visible subscriber.
