//! Single-level folder operations.

pub use std::fs::{
    create_dir as create_folder, create_dir_all as create_folder_all, read_dir, DirEntry as FolderEntry,
    ReadDir as ReadFolder,
};
