# fs-tools

Extended `std::fs` for the rust-boost workspace.

## Layout

| Module        | Role                                                                  |
|---------------|-----------------------------------------------------------------------|
| `file`        | `File`, `OpenOptions`, `exists`                                       |
| `folder`      | `read_dir`, `create_folder`, `create_folder_all`, `FolderEntry`, `ReadFolder` |
| `metadata`    | `metadata`, `symlink_metadata`, `Metadata`, `FileType`                |
| `link`        | `canonicalize`, `hard_link`, `symlink`, `read_link`                   |
| `read_write`  | `read`, `write`, `read_to_string`, `copy`                             |
| `remove`      | `remove_file`, `remove_folder`, `remove_folder_all`                   |
| `rename`      | `rename`                                                              |
| `publish`     | `AtomicPublisher`, `publish_bytes`, overwrite policy, staging cleanup |
| `permissions` | `set_permissions`, `Permissions`                                      |
| `temp`        | `create_folder`, `create_file`, `NamedFile`, `configure_folder`       |
| `walk`        | `Walker`, `Entry`, `Filter`, `ErrorPolicy`, `Iter`                    |
| `async_walk`  | `Walker`, `Filter` (`async` feature)                                  |

Crate-root re-exports mirror `std::fs` with folder-oriented names where applicable. Extensions live under `temp`, `walk`, and `async_walk`.

A walk applies its predicate before an entry is yielded, so rejecting a folder with `Filter::IgnoreFolder` prunes its subtree instead of walking it and discarding the result. `sort_by_file_name` makes the traversal order reproducible, `follow_links` reports a link that resolves to a folder on its own branch as a loop, and `ErrorPolicy::Strict` ends the iteration at the first error instead of continuing with an incomplete tree.

Publishing stages bytes in the destination folder and then publishes them. `RefuseExisting` publishes by linking, so a destination that appeared while the file was staged is refused without being replaced. A failed publish removes the staging file, and a failure after publishing means the destination is visible but not known to be durable. The destination folder is flushed on platforms that hand out a folder handle.

Temporary paths take their candidate name from the host entropy source of `random`, so a candidate is not predictable from a timestamp or a counter. Randomness only names a candidate: every temporary file is still created with `create_new` and every temporary folder with an atomic create, and a collision is retried. When the host has no usable entropy source, the candidate fails instead of falling back to a predictable name.

On WASI, call `temp::configure_folder` or `temp::create_folder_in` with a preopened writable folder before creating temporary paths.
