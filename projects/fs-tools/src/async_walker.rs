use std::{
    future::Future,
    path::{Path, PathBuf},
    pin::Pin,
    task::{Context, Poll},
};

use async_channel::Receiver;
use futures_core::Stream;
use pin_project_lite::pin_project;

use crate::{DirEntry, WalkError, Walker};

/// Controls how [`AsyncWalker`] handles an entry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Filtering {
    /// Ignore the current entry.
    Ignore,
    /// Ignore the current entry and do not traverse its children.
    IgnoreDir,
    /// Continue traversal normally.
    Continue,
}

pin_project! {
    /// A `Stream` of directory entries generated from recursively traversing a tree.
    ///
    /// The root directory itself is not yielded, matching `async-walkdir` behavior.
    pub struct AsyncWalker {
        root: PathBuf,
        #[pin]
        receiver: Receiver<Result<DirEntry, WalkError>>,
    }
}

impl AsyncWalker {
    /// Create an async walker rooted at `root`.
    pub fn new(root: impl AsRef<Path>) -> Self {
        let root = root.as_ref().to_path_buf();
        let (sender, receiver) = async_channel::bounded(64);
        let walk_root = root.clone();
        std::thread::spawn(move || {
            for item in Walker::new(walk_root).min_depth(1) {
                if sender.send_blocking(item).is_err() {
                    break;
                }
            }
        });
        Self { root, receiver }
    }

    /// Filter entries before they are yielded.
    pub fn filter<F, Fut>(self, mut filter: F) -> Self
    where
        F: FnMut(DirEntry) -> Fut + Send + 'static,
        Fut: Future<Output = Filtering> + Send + 'static,
    {
        let root = self.root;
        let walk_root = root.clone();
        let (sender, receiver) = async_channel::bounded(64);
        std::thread::spawn(move || {
            for item in Walker::new(walk_root).min_depth(1) {
                match item {
                    Ok(entry) => {
                        if futures_lite::future::block_on(filter(entry.clone())) != Filtering::Continue {
                            continue;
                        }
                        if sender.send_blocking(Ok(entry)).is_err() {
                            break;
                        }
                    }
                    Err(err) => {
                        if sender.send_blocking(Err(err)).is_err() {
                            break;
                        }
                        break;
                    }
                }
            }
        });
        Self { root, receiver }
    }
}

impl Stream for AsyncWalker {
    type Item = Result<DirEntry, WalkError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().receiver.poll_next(cx)
    }
}
