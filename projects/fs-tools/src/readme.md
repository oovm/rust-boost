# fs-tools

Extended `std::fs` for the rust-boost workspace.

## Layout

| Module        | Role                                                              |
|---------------|-------------------------------------------------------------------|
| `file`        | `File`, `OpenOptions`, `exists`                                   |
| `dir`         | `read_dir`, `create_dir`, `create_dir_all`, `DirEntry`, `ReadDir` |
| `metadata`    | `metadata`, `symlink_metadata`, `Metadata`, `FileType`            |
| `link`        | `canonicalize`, `hard_link`, `symlink`, `read_link`               |
| `read_write`  | `read`, `write`, `read_to_string`, `copy`                         |
| `remove`      | `remove_file`, `remove_dir`, `remove_dir_all`                     |
| `rename`      | `rename`                                                          |
| `permissions` | `set_permissions`, `Permissions`                                  |
| `temp`        | `create_dir`, `create_file`, `NamedFile`, `configure_dir`         |
| `walk`        | `Walker`, `Entry`, `Error`, `Iter`                                |
| `async_walk`  | `Walker`, `Filter` (`async` feature)                              |

Crate-root re-exports mirror `std::fs`. Extensions live under `temp`, `walk`, and `async_walk`.

On WASI, call `temp::configure_dir` or `temp::create_dir_in` with a preopened writable directory before creating
temporary paths.
