# boost-log

Structured log and event facade: `LogEvent`, spans, filters, sinks, and global dispatch — without terminal styling or diagnostic models.

Human-visible stderr rendering moved to [`boost-console`](../boost-console/readme.md). Use that crate (or `boost::console`) for `render` and `install_global_subscriber`.

Repository context: [boost README](../../readme.md).

## Example

```rust
use boost_log::{Level, LogSink, VecSink, event, set_global_sink};

let mut sink = VecSink::new();
set_global_sink(Box::new(sink));
event!(Info, "worker", message = "ready");
```

## Related packages

- [`boost-console`](../boost-console/readme.md) — stderr rendering and default subscriber
- [`boost-diagnostic`](../boost-diagnostic/readme.md) — optional log projection for diagnostics

Dispatch model, async sinks, filters, and task identity: [`src/readme.md`](src/readme.md).
