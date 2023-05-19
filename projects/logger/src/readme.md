# `logger`

Workspace-wide structured log and event facade.

`logger` owns `LogEvent`, span context, filters, sinks, and global dispatch. It does not depend on `diagnostic`, `console`, or `terminal`. Domain crates can emit structured events through `logger::event!` without importing parser or diagnostic models.

When no global sink is installed, events are silently dropped and a drop counter is incremented. Sink callbacks never run while holding the global registration lock.

`AsyncSink` puts a worker thread in front of any sink with a fixed queue capacity. A full queue drops and counts instead of blocking the emitting thread, `flush` waits until everything queued before it has been written, and `close` drains the queue and returns the final flush outcome. `flush_global_sink` reports the installed sink's outcome or `None` when no sink is installed, and installing a sink again replaces the previous one.

A field keeps its type when it has one. `field_value!` converts booleans, integers, floats, strings, characters, and existing `FieldValue`s without stringifying them, and JSON output writes numbers and booleans unquoted. Any other type must be passed through `FieldValue::debug`, so a value logger cannot represent fails to compile instead of becoming a `Debug` string.

`Filter` holds a global level plus optional target directives. `Filter::parse` reads the comma-separated form (`info`, `oak=debug`, a bare `oak` for `trace`), the longest matching target wins at a `::` boundary, and only target directives leave every unnamed target dropped. `Filter::from_env` reads a variable named by the caller and returns an error for an invalid directive or a value that is not Unicode instead of falling back to a default level.

A task identity is carried explicitly. `TaskContext::new` starts a task, `child` descends from it, and `TaskContext::log` builds an event that records the task and its fields. Nothing is read from the thread, so a task that resumes on another thread keeps its own identity and cannot pick up the identity of a task that ran there earlier. `with_scope` and `enter(span!(..))` are thread scopes: they follow the thread and are never recorded as a task. A task that can cross an await point emits through its context instead of entering a span.
