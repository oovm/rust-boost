# `logger`

Workspace-wide structured log and event facade.

`logger` owns `LogEvent`, span context, filters, sinks, and global dispatch. It does not depend on `diagnostic`, `console`, or `terminal`. Domain crates can emit structured events through `logger::event!` without importing parser or diagnostic models.

When no global sink is installed, events are silently dropped and a drop counter is incremented. Sink callbacks never run while holding the global registration lock.
