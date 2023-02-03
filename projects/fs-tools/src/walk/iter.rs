use std::{
    fs::{self, FileType},
    path::{Path, PathBuf},
};

use super::{Entry, Error, Walker};

/// Iterator over paths discovered by [`Walker`].
#[derive(Debug)]
pub struct Iter {
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

impl Iter {
    pub(crate) fn new(walker: Walker) -> Self {
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

    fn push_children(&mut self, path: &Path, depth: usize) -> Result<(), Error> {
        if depth >= self.max_depth {
            return Ok(());
        }

        let read_dir = fs::read_dir(path).map_err(|err| Error::io(path, depth, err))?;
        let mut children = Vec::new();
        for entry in read_dir {
            let entry = entry.map_err(|err| Error::io(path, depth + 1, err))?;
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

    fn metadata_for(&mut self, path: &Path, depth: usize) -> Result<(PathBuf, FileType), Error> {
        if self.follow_links {
            let metadata = fs::metadata(path).map_err(|err| Error::io(path, depth, err))?;
            if metadata.file_type().is_symlink() {
                let canonical = fs::canonicalize(path).map_err(|err| Error::io(path, depth, err))?;
                if self.seen_links.iter().any(|seen| seen == &canonical) {
                    let ancestor = self.seen_links.last().cloned().unwrap_or_else(|| canonical.clone());
                    return Err(Error::loop_at(path, depth, ancestor));
                }
                self.seen_links.push(canonical);
            }
            Ok((path.to_path_buf(), metadata.file_type()))
        }
        else {
            let file_type = fs::symlink_metadata(path).map_err(|err| Error::io(path, depth, err))?.file_type();
            Ok((path.to_path_buf(), file_type))
        }
    }
}

impl Iterator for Iter {
    type Item = Result<Entry, Error>;

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
                return Some(Ok(Entry::new(path, pending.depth, file_type)));
            }
        }

        self.finished = true;
        None
    }
}
