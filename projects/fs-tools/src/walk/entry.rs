use std::{
    fs::FileType,
    path::{Path, PathBuf},
};

/// A path discovered while recursively walking a folder tree.
#[derive(Debug, Clone)]
pub struct Entry {
    path: PathBuf,
    depth: usize,
    file_type: FileType,
}

impl Entry {
    /// Returns the full path of this entry.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the depth of this entry relative to the walk root.
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// Returns the file name of this entry.
    pub fn file_name(&self) -> &std::ffi::OsStr {
        self.path.file_name().unwrap_or_default()
    }

    /// Returns the file type of this entry.
    pub fn file_type(&self) -> FileType {
        self.file_type
    }

    /// Returns whether this entry is a folder.
    pub fn is_folder(&self) -> bool {
        self.file_type.is_dir()
    }

    /// Returns whether this entry is a file.
    pub fn is_file(&self) -> bool {
        self.file_type.is_file()
    }

    /// Returns whether this entry is a symlink.
    pub fn is_symlink(&self) -> bool {
        self.file_type.is_symlink()
    }

    pub(crate) fn new(path: PathBuf, depth: usize, file_type: FileType) -> Self {
        Self { path, depth, file_type }
    }
}
