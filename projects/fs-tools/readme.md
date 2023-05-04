# fs-tools

Extended `std::fs` for the rust-boost workspace.

Use crate-root APIs where you would use `std::fs`. Recursive traversal is `fs_tools::walk::Walker`, which can prune folders, sort the entries of a folder, and decide how an error affects the rest of the iteration. Temporary paths live in `fs_tools::temp`. `fs_tools::publish` publishes a file without leaving its staging file behind. `fs_tools::watch` observes file changes with a bounded event queue and an explicit rescan when events are dropped. The default native backend uses `ReadDirectoryChangesW` on Windows. Other hosts reject it with `Unsupported` and require the explicit polling backend. On WASI call `fs_tools::temp::configure_folder` or `create_folder_in` with a preopened writable folder before creating temporary files.
