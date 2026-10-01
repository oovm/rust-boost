use std::{
    future::Future,
    path::{Path, PathBuf},
    pin::Pin,
    task::{Context, Poll},
};

use async_channel::Receiver;
use futures_core::Stream;
use pin_project_lite::pin_project;

use crate::walk::{Entry, Error, Walker as SyncWalker};

use super::Filter;

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
            for item in SyncWalker::new(walk_root).min_depth(1) {
                match item {
                    Ok(entry) => {
                        if futures_lite::future::block_on(predicate(entry.clone())) != Filter::Continue {
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

impl Stream for Walker {
    type Item = Result<Entry, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.project().receiver.poll_next(cx)
    }
}
