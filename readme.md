# Rust Boost

[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](License.md)

[docs.rs](https://docs.rs/boost) · [repository](https://github.com/oovm/rust-boost)

## 💡 What is boost?

**boost** is a family of Rust libraries that extend and correct gaps in the standard library without pulling in a
third-party ecosystem. Each capability is a small, orthogonal crate — source identity, structured logging, diagnostics,
terminal output, filesystem helpers, deterministic randomness, and test harnesses — that you can depend on alone or
through a single feature-gated entry crate named `boost`.

boost is **not** a logging framework, a test runner distribution, or a protocol stack. It owns stable data models and
host-facing behavior contracts. Downstream products (Oak, Acorn, language servers, CLIs) compose these facts into their
own protocols and UIs.

Design constraints:

- Runtime crates target **std / core / alloc** and other boost members only.
- The `boost` facade exposes capabilities as `boost::log`, `boost::console`, `boost::diagnostic`, … — never as
  re-exported external crate names.
- LSP, SVG, and other product-specific adapters live outside this repository.

## 🚀 Getting started

### 1. Agent / prompt (recommended)

> Add `boost` with `default-features = false` and enable only `log` and `console`. Emit structured events through
> `boost::log::event!`. Install the default human-visible stderr subscriber with
> `boost::console::install_global_subscriber()`. Do not add `tracing`, `log`, or `serde` for this path.

### 2. Manual install

```toml
[dependencies]
boost = { version = "0.1", default-features = false, features = ["log", "console", "diagnostic"] }
```

```rust
use boost::console::install_global_subscriber;
use boost::log::{Level, event};

fn main() {
    install_global_subscriber();
    event!(Info, "app.startup", message = "ready");
}
```

Prefer a single capability? Depend on the matching `boost-*` crate directly (`boost-log`, `boost-console`,
`boost-diagnostic`, …) with the same types and no extra indirection.

## ✨ Highlights

| Capability | Crate              | Role                                                             |
|------------|--------------------|------------------------------------------------------------------|
| Facade     | `boost`            | Feature-gated `boost::…` modules; empty default features         |
| Source     | `boost-source`     | Stable file identity, immutable snapshots, byte/line coordinates |
| Log        | `boost-log`        | `LogEvent`, spans, filters, sinks — no terminal or styling       |
| Console    | `boost-console`    | `fmt` styling module, stderr rendering, default subscriber       |
| Terminal   | `boost-terminal`   | TTY/ANSI capability and device facts                             |
| Diagnostic | `boost-diagnostic` | Structured problems: codes, labels, severity, collections        |
| Test       | `boost-test`       | Cases, fixtures, assertions, structured `TestResult`             |
| Random     | `boost-random`     | Deterministic streams, sampling, host entropy interfaces         |
| FS         | `boost-fs`         | Walk, watch, temp files, atomic publish                          |

Macros (`boost-test-macro`, `boost-diagnostic-macro`, …) compile against `syn` and generate code that references the
runtime crates above.

## 📊 How boost compares

|                    | boost                                   | Typical ecosystem crates                        |
|--------------------|-----------------------------------------|-------------------------------------------------|
| Dependency closure | std + boost members                     | serde, tracing, url, TLS stacks, async runtimes |
| Capability shape   | One fact owner per crate                | Mixed transport + serde + UI in one package     |
| Facade             | Opt-in `boost::module` namespaces       | Often a single prelude re-exporting everything  |
| Diagnostics        | Structured facts; rendering is separate | Often tied to one output backend                |

## 🧪 Examples

Each published crate ships a focused `readme.md` with import paths, feature flags, and a minimal usage block:

- [`boost`](projects/boost/readme.md) — facade features and module map
- [`boost-console`](projects/boost-console/readme.md) — stderr rendering and subscriber
- [`boost-log`](projects/boost-log/readme.md) — events, filters, async sinks
- [`boost-diagnostic`](projects/boost-diagnostic/readme.md) — diagnostic model and projections
- [`boost-test`](projects/boost-test/readme.md) — cases, fixtures, assertions

## 📜 License

Apache-2.0 — see [License.md](License.md).
