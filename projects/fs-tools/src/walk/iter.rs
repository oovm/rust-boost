use std::{
    fs::{self, FileType},
    path::{Path, PathBuf},
    rc::Rc,
};

use super::{Entry, Error, ErrorPolicy, Filter, Walker};

/// A predicate that decides what a walk yields and what it traverses.
type EntryFilter = Box<dyn FnMut(&Entry) -> Filter>;

/// Iterator over paths discovered by [`Walker`].
pub struct Iter {
    root: PathBuf,
    min_depth: usize,
    max_depth: usize,
    follow_links: bool,
    sort_by_file_name: bool,
    error_policy: ErrorPolicy,
    filter: Option<EntryFilter>,
    stack: Vec<PendingEntry>,
    started: bool,
    finished: bool,
}

impl std::fmt::Debug for Iter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Iter")
            .field("root", &self.root)
            .field("min_depth", &self.min_depth)
            .field("max_depth", &self.max_depth)
            .field("follow_links", &self.follow_links)
            .field("sort_by_file_name", &self.sort_by_file_name)
            .field("filtered", &self.filter.is_some())
            .field("pending", &self.stack.len())
            .field("finished", &self.finished)
            .finish()
    }
}

/// Folders on the branch that leads to an entry, identified after following links.
///
/// A link is only a loop when it resolves to a folder that already contains the entry, so the
/// branch is compared instead of every folder seen so far. Two sibling links to the same folder are
/// therefore not reported as a loop.
#[derive(Debug)]
struct Ancestor {
    canonical: PathBuf,
    parent: Option<Rc<Ancestor>>,
}

impl Ancestor {
    /// Returns the branch folder that `canonical` resolves to, when it is already on the branch.
    fn matching(&self, canonical: &Path) -> Option<&Path> {
        let mut current = Some(self);
        while let Some(node) = current {
            if node.canonical == canonical {
                return Some(&node.canonical);
            }
            current = node.parent.as_deref();
        }
        None
    }
}

#[derive(Debug)]
struct PendingEntry {
    path: PathBuf,
    depth: usize,
    ancestors: Option<Rc<Ancestor>>,
}

/// An entry whose file type and, while links are followed, canonical folder are known.
struct Resolved {
    path: PathBuf,
    file_type: FileType,
    canonical: Option<PathBuf>,
}

impl Iter {
    pub(crate) fn new(walker: Walker) -> Self {
        Self::build(walker, None)
    }

    pub(crate) fn filtered<F>(walker: Walker, predicate: F) -> Self
    where
        F: FnMut(&Entry) -> Filter + 'static,
    {
        Self::build(walker, Some(Box::new(predicate)))
    }

    fn build(walker: Walker, filter: Option<EntryFilter>) -> Self {
        Self {
            root: walker.root,
            min_depth: walker.min_depth,
            max_depth: walker.max_depth,
            follow_links: walker.follow_links,
            sort_by_file_name: walker.sort_by_file_name,
            error_policy: walker.error_policy,
            filter,
            stack: Vec::new(),
            started: false,
            finished: false,
        }
    }

    /// Report an error according to the configured policy.
    fn failure(&mut self, error: Error) -> Option<Result<Entry, Error>> {
        if self.error_policy == ErrorPolicy::Strict {
            self.finished = true;
        }
        Some(Err(error))
    }

    fn push_children(&mut self, path: &Path, depth: usize, ancestors: Option<Rc<Ancestor>>) -> Result<(), Error> {
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
            children.push(PendingEntry { path: child_path, depth: depth + 1, ancestors: ancestors.clone() });
        }

        if self.sort_by_file_name {
            children.sort_by(|left, right| left.path.file_name().cmp(&right.path.file_name()));
        }
        children.reverse();
        self.stack.extend(children);
        Ok(())
    }

    /// Read the file type of an entry, and the canonical folder it belongs to while links are
    /// followed.
    fn resolve(&self, pending: &PendingEntry) -> Result<Resolved, Error> {
        let path = pending.path.as_path();
        let link_metadata = fs::symlink_metadata(path).map_err(|err| Error::io(path, pending.depth, err))?;
        if !self.follow_links {
            return Ok(Resolved { path: pending.path.clone(), file_type: link_metadata.file_type(), canonical: None });
        }

        // Following links is what makes a resolution ambiguous, so the canonical path is computed for
        // every folder on the branch and compared when a link is met.
        let canonical = fs::canonicalize(path).map_err(|err| Error::io(path, pending.depth, err))?;
        if !link_metadata.file_type().is_symlink() {
            return Ok(Resolved { path: pending.path.clone(), file_type: link_metadata.file_type(), canonical: Some(canonical) });
        }

        if let Some(ancestor) = pending.ancestors.as_deref().and_then(|branch| branch.matching(&canonical)) {
            return Err(Error::loop_at(path, pending.depth, ancestor.to_path_buf()));
        }

        let followed = fs::metadata(path).map_err(|err| Error::io(path, pending.depth, err))?;
        Ok(Resolved { path: pending.path.clone(), file_type: followed.file_type(), canonical: Some(canonical) })
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
            self.stack.push(PendingEntry { path: self.root.clone(), depth: 0, ancestors: None });
        }

        while let Some(pending) = self.stack.pop() {
            let resolved = match self.resolve(&pending) {
                Ok(resolved) => resolved,
                Err(error) => return self.failure(error),
            };

            let entry = Entry::new(resolved.path, pending.depth, resolved.file_type);
            let decision = match self.filter.as_mut() {
                Some(filter) => filter(&entry),
                None => Filter::default(),
            };

            if entry.is_folder() && decision != Filter::IgnoreFolder {
                let ancestors = match resolved.canonical {
                    Some(canonical) => Some(Rc::new(Ancestor { canonical, parent: pending.ancestors.clone() })),
                    None => pending.ancestors.clone(),
                };
                if let Err(error) = self.push_children(entry.path(), pending.depth, ancestors) {
                    return self.failure(error);
                }
            }

            if decision == Filter::Continue && pending.depth >= self.min_depth {
                return Some(Ok(entry));
            }
        }

        self.finished = true;
        None
    }
}
