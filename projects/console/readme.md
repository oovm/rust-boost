# `console`

Human-visible console subscriber and generic terminal styling.

`console` depends on `logger` and `terminal`, not on `diagnostic`. It provides color, paint, layout helpers, and `install_global_subscriber` to attach the default stderr subscriber to the global `logger` facade.

Domain crates should use `logger` for structured events, spans, and sinks. Use `console` only for terminal styling or when installing the default human-visible subscriber.
