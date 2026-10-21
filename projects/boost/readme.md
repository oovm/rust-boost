# boost

Single entry point for the boost standard-library enhancement family. Each enabled feature opens a `boost::…` module that re-exports one runtime crate — same types, no duplicate wrappers.

Product overview and design constraints: [repository README](../../readme.md).

## Features

Default features are **empty**. Enable only what you need:

| Feature | Module | Underlying crate |
|---------|--------|------------------|
| `source` | `boost::source` | `boost-source` |
| `random` | `boost::random` | `boost-random` |
| `fmt` | `boost::fmt` | `boost-console` (`fmt` module only) |
| `terminal` | `boost::terminal` | `boost-terminal` |
| `console` | `boost::console` | `boost-console` (full console surface) |
| `fs` | `boost::fs` | `boost-fs` |
| `log` | `boost::log` | `boost-log` |
| `diagnostic` | `boost::diagnostic` | `boost-diagnostic` |
| `test` | `boost::test` | `boost-test` |

`fmt` enables only the styling submodule inside `boost-console`. `console` enables the full console surface (render, sinks, subscriber) and implies `log` and `terminal`. `diagnostic` pulls `boost-source` through `boost-diagnostic`.

## Example

```toml
[dependencies]
boost = { version = "0.1", default-features = false, features = ["log", "console"] }
```

```rust
use boost::console::install_global_subscriber;
use boost::log::{Level, event};

fn main() {
    install_global_subscriber();
    event!(Info, "worker", message = "started");
}
```

## Direct crates

Libraries that expose only one concern should depend on `boost-log`, `boost-console`, or another member crate directly. The facade is for applications and integration tests that want one manifest line and explicit feature toggles.

## Related packages

- [`boost-console`](../boost-console/readme.md) — human-visible stderr output (depends on `boost-log`)
- [`boost-diagnostic`](../boost-diagnostic/readme.md) — structured diagnostic facts
- [`boost-test`](../boost-test/readme.md) — correctness harness and assertions
