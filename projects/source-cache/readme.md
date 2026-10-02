# `source-cache`

**Deprecated.** New code must use `source` for identity, snapshots, and byte access, and `diagnostic::terminal::SourceCache` for terminal rendering. This crate remains only until `diagnostic-lsp` and `diagnostic-svg` finish migration.

## Responsibilities

- `provenance` exports stable `SourceRef`, byte ranges, container member paths, object references, and semantic paths shared by `diagnostic`, renderers, and upstream parsers.
- `SourceCache` stores loaded text and path metadata when the `std` feature is enabled.
- `SourceProvider` resolves `SourceID` values for terminal, LSP, and SVG renderers.

Renderers depend on `SourceProvider`, not on ad hoc hash construction. Build identifiers through `SourcePath::source_id`, `SourceID::from_path`, or `SourceCache` loaders.
