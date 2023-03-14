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
| `walk`        | `Walker`, `Entry`, `Error`, `Iter`                                    |
| `async_walk`  | `Walker`, `Filter` (`async` feature)                                  |

Crate-root re-exports mirror `std::fs` with folder-oriented names where applicable. Extensions live under `temp`, `walk`, and `async_walk`.

On WASI, call `temp::configure_folder` or `temp::create_folder_in` with a preopened writable folder before creating temporary paths.
