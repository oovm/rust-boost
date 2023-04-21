use std::{
    future::Future,
    path::{Path, PathBuf},
    pin::Pin,
    task::{Context, Poll},
};

use async_channel::Receiver;
use futures_core::Stream;
use pin_project_lite::pin_project;

use crate::walk::{Entry, Error, Filter, Walker as SyncWalker};

pin_project! {
    /// A `Stream` of paths discovered by recursively walking a folder tree.
    ///
    /// The root folder itself is not yielded.
    pub struct Walker {
        root: PathBuf,
        #[pin]
        receiver: Receiver<Result<Entry, Error>>,
    }
}

impl Walker {
    /// Create a walker rooted at `root`.
    pub fn new(root: impl AsRef<Path>) -> Self {
        let root = root.as_ref().to_path_buf();
        let (sender, receiver) = async_channel::bounded(64);
        let walk_root = root.clone();
        std::thread::spawn(move || {
            for item in SyncWalker::new(walk_root).min_depth(1) {
                if sender.send_blocking(item).is_err() {
                    break;
                }
            }
        });
        Self { root, receiver }
    }

    /// Filter entries before they are yielded.
    ///
    /// The predicate runs on the traversal thread, so a folder that it reports as
    /// [`Filter::IgnoreFolder`] is pruned instead of being visited and discarded.
    pub fn filter<F, Fut>(self, predicate: F) -> Self
    where
        F: FnMut(Entry) -> Fut + Send + 'static,
        Fut: Future<Output = Filter> + Send + 'static,
    {
        let root = self.root;
        let walk_root = root.clone();
        let (sender, receiver) = async_channel::bounded(64);
        std::thread::spawn(move || {
            let mut predicate = predicate;
            let walker = SyncWalker::new(walk_root).min_depth(1).filter(move |entry| futures_lite::future::block_on(predicate(entry.clone())));
            for item in walker {
                if sender.send_blocking(item).is_err() {
                    break;
                }
            }
        });
        Self { root, receiver }
    }
}

impl Stream for Walker {
    type Item = Result<Entry, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().receiver.poll_next(cx)
    }
}
