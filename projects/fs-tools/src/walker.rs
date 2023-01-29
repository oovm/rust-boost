use std::{
    fs::{self, FileType},
    path::{Path, PathBuf},
};

use crate::error::WalkError;

/// A directory entry produced by [`Walker`].
#[derive(Debug, Clone)]
pub struct DirEntry {
    path: PathBuf,
    depth: usize,
    file_type: FileType,
}

impl DirEntry {
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

    /// Returns whether this entry is a directory.
    pub fn is_dir(&self) -> bool {
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
}

/// A builder for recursively walking a directory tree.
#[derive(Debug, Clone)]
pub struct Walker {
    root: PathBuf,
    min_depth: usize,
    max_depth: usize,
    follow_links: bool,
}

impl Walker {
    /// Create a walker rooted at `root`.
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
            min_depth: 0,
            max_depth: usize::MAX,
            follow_links: false,
        }
    }

    /// Returns the configured root path.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Set the minimum depth of entries yielded by the iterator.
    pub fn min_depth(self, depth: usize) -> Self {
        Self { min_depth: depth, ..self }
    }

    /// Set the maximum depth of entries yielded by the iterator.
    pub fn max_depth(self, depth: usize) -> Self {
        Self { max_depth: depth, ..self }
    }

    /// Follow symbolic links while walking directories.
    pub fn follow_links(self, yes: bool) -> Self {
        Self { follow_links: yes, ..self }
    }
}

impl IntoIterator for Walker {
    type Item = Result<DirEntry, WalkError>;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self)
    }
}

/// The iterator returned by [`Walker`].
#[derive(Debug)]
pub struct IntoIter {
    root: PathBuf,
    min_depth: usize,
    max_depth: usize,
    follow_links: bool,
    stack: Vec<PendingEntry>,
    seen_links: Vec<PathBuf>,
    started: bool,
    finished: bool,
}

#[derive(Debug)]
struct PendingEntry {
    path: PathBuf,
    depth: usize,
}

impl IntoIter {
    fn new(walker: Walker) -> Self {
        Self {
            root: walker.root,
            min_depth: walker.min_depth,
            max_depth: walker.max_depth,
            follow_links: walker.follow_links,
            stack: Vec::new(),
            seen_links: Vec::new(),
            started: false,
            finished: false,
        }
    }

    fn push_children(&mut self, path: &Path, depth: usize) -> Result<(), WalkError> {
        if depth >= self.max_depth {
            return Ok(());
        }

        let read_dir = fs::read_dir(path).map_err(|err| WalkError::io(path, depth, err))?;
        let mut children = Vec::new();
        for entry in read_dir {
            let entry = entry.map_err(|err| WalkError::io(path, depth + 1, err))?;
            let child_path = entry.path();
            if child_path.file_name().is_some_and(|name| name == "." || name == "..") {
                continue;
            }
            children.push(PendingEntry { path: child_path, depth: depth + 1 });
        }
        children.reverse();
        self.stack.extend(children);
        Ok(())
    }

    fn metadata_for(&mut self, path: &Path, depth: usize) -> Result<(PathBuf, FileType), WalkError> {
        if self.follow_links {
            let metadata = fs::metadata(path).map_err(|err| WalkError::io(path, depth, err))?;
            if metadata.file_type().is_symlink() {
                let canonical = fs::canonicalize(path).map_err(|err| WalkError::io(path, depth, err))?;
                if self.seen_links.iter().any(|seen| seen == &canonical) {
                    let ancestor = self.seen_links.last().cloned().unwrap_or_else(|| canonical.clone());
                    return Err(WalkError::loop_at(path, depth, ancestor));
                }
                self.seen_links.push(canonical);
            }
            Ok((path.to_path_buf(), metadata.file_type()))
        }
        else {
            let file_type = fs::symlink_metadata(path).map_err(|err| WalkError::io(path, depth, err))?.file_type();
            Ok((path.to_path_buf(), file_type))
        }
    }
}

impl Iterator for IntoIter {
    type Item = Result<DirEntry, WalkError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        if !self.started {
            self.started = true;
            self.stack.push(PendingEntry { path: self.root.clone(), depth: 0 });
        }

        while let Some(pending) = self.stack.pop() {
            let (path, file_type) = match self.metadata_for(&pending.path, pending.depth) {
                Ok(value) => value,
                Err(err) => return Some(Err(err)),
            };

            if file_type.is_dir() {
                if let Err(err) = self.push_children(&path, pending.depth) {
                    return Some(Err(err));
                }
            }

            if pending.depth >= self.min_depth {
                return Some(Ok(DirEntry { path, depth: pending.depth, file_type }));
            }
        }

        self.finished = true;
        None
    }
}
