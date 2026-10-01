# `source-cache`

Text source storage and lookup for renderers.

## Responsibilities

- `SourceCache` stores loaded text and path metadata.
- `SourceProvider` resolves `SourceID` values for terminal, LSP, and SVG renderers.
- `SourceRef` is the stable cross-crate source identity contract.

Renderers depend on `SourceProvider`, not on ad hoc hash construction. Build identifiers through `SourcePath::source_id`, `SourceID::from_path`, or `SourceCache` loaders.
