# boost-console

Terminal text styling and human-visible console output for structured log events.

- **`boost_console::fmt`** — colors, paint, style, stream-aware formatting (std only; no log dependency)
- **Console surface** — plain/styled `LogEvent` rendering, stderr sinks, `install_global_subscriber` (requires `console` feature)

Repository context: [boost README](../../readme.md).

## Features

| Feature | What you get |
|---------|----------------|
| `fmt` (via `default-features = false`) | `boost_console::fmt::*` only |
| `console` (default) | `fmt` + render/sink/subscriber (`boost-log`, `boost-terminal`) |

## When to use

| Need | Import |
|------|--------|
| ANSI colors, paint, display width for diagnostics or CLIs | `boost_console::fmt` or `boost::fmt` |
| Render `LogEvent` to stderr / install default subscriber | `boost_console` with `console` or `boost::console` |
| Emit events, filters, file sinks | [`boost-log`](../boost-log/readme.md) |

## Example

Styling only:

```rust
use boost_console::fmt::{Color, Paint};

let line = Paint::green("ready").to_string();
```

Full console (facade):

```toml
[dependencies]
boost = { version = "0.1", default-features = false, features = ["log", "console"] }
```

```rust
use boost::console::install_global_subscriber;
use boost::log::event;

fn main() {
    install_global_subscriber();
    event!(Info, "worker", message = "started");
}
```

## API surface (console feature)

| Item | Purpose |
|------|---------|
| `render` / `render_plain` | One `LogEvent` as a terminal line |
| `StderrSubscriberSink` / `StderrFallbackSink` | `LogSink` implementations |
| `install_global_subscriber` | Default styled stderr sink on the global logger |

Module boundaries: [`src/readme.md`](src/readme.md). Styling types live under [`src/fmt/`](src/fmt/mod.rs).

## Related packages

- [`boost-log`](../boost-log/readme.md) — event model
- [`boost-diagnostic`](../boost-diagnostic/readme.md) — terminal rendering uses `boost_console::fmt`
- [`boost`](../boost/readme.md) — `fmt` and `console` features
