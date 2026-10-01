# fs-tools

Extended `std::fs` for the rust-boost workspace.

Use crate-root APIs where you would use `std::fs`. Recursive traversal is `fs_tools::walk::Walker`. Temporary paths live in `fs_tools::temp`. On WASI call `fs_tools::temp::configure_folder` or `create_folder_in` with a preopened writable folder before creating temporary files.
