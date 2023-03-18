# `source`

Stable source identity, immutable byte snapshots, and UTF-8 line coordinates.

`source` answers how input is referenced, how bytes are read at a fixed revision, and how byte offsets map to line/column views. It does not parse formats, fetch URLs, own diagnostics, or encode container or IR semantics.

See the workspace design contract for the frozen public API.
