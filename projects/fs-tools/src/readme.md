# fs-tools

Extended `std::fs` for the rust-boost workspace.

## Layout

| Module | Role |
|--------|------|
| `file` | `File`, `OpenOptions`, `exists` |
| `dir` | `read_dir`, `create_dir`, `create_dir_all`, `DirEntry`, `ReadDir` |
| `metadata` | `metadata`, `symlink_metadata`, `Metadata`, `FileType` |
| `link` | `canonicalize`, `hard_link`, `symlink`, `read_link` |
| `read_write` | `read`, `write`, `read_to_string`, `copy` |
| `remove` | `remove_file`, `remove_dir`, `remove_dir_all` |
| `rename` | `rename` |
| `permissions` | `set_permissions`, `Permissions` |
| `walk` | `Walker`, `Entry`, `Error`, `Iter` |
| `async_walk` | `Walker`, `Filter` (`async` feature) |

Crate-root re-exports mirror `std::fs`. Extensions live only under `walk` and `async_walk`.
