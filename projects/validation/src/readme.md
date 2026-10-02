# `validation`

**Deprecated outside migration inventory.** This crate provides `Validation<T, E>` and fatal-plus-diagnostics result wrapping. It is not part of the unified `diagnostic` contract.

New code should return domain results and hold `DiagnosticSet` explicitly when multiple structured diagnostics are needed. Do not introduce new dependencies on this crate until a removal plan is published.
