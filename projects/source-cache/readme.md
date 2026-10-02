# `source-cache`

Source identity, provenance primitives, and optional text cache.

## Responsibilities

- `provenance` exports stable `SourceRef`, byte ranges, container member paths, object references, and semantic paths shared by `diagnostic`, renderers, and upstream parsers.
- `SourceCache` stores loaded text and path metadata when the `std` feature is enabled.
- `SourceProvider` resolves `SourceID` values for terminal, LSP, and SVG renderers.

Renderers depend on `SourceProvider`, not on ad hoc hash construction. Build identifiers through `SourcePath::source_id`, `SourceID::from_path`, or `SourceCache` loaders.
