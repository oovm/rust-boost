//! Watcher construction options.

use std::time::Duration;

use super::WatchError;

/// Default number of events buffered between the backend and the callback.
pub const DEFAULT_QUEUE_CAPACITY: usize = 4096;

/// Lower bound accepted for a polling interval.
pub const MIN_POLL_INTERVAL: Duration = Duration::from_millis(10);

/// How a watcher observes the file system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WatchBackend {
    /// Use the host's native change notification backend.
    Native,
    /// Compare directory snapshots at a fixed interval.
    ///
    /// Polling only reports the net state change between two samples. It never reports
    /// [`super::WatchEvent::Renamed`] and must be selected explicitly.
    Polling {
        /// Time between snapshots.
        interval: Duration,
    },
}

/// Options used to create a [`Watcher`](super::Watcher).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WatchOptions {
    queue_capacity: usize,
    backend: WatchBackend,
}

impl Default for WatchOptions {
    fn default() -> Self {
        Self { queue_capacity: DEFAULT_QUEUE_CAPACITY, backend: WatchBackend::Native }
    }
}

impl WatchOptions {
    /// Create options with the default capacity and the native backend.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the number of events buffered before overflow handling starts.
    pub fn queue_capacity(mut self, capacity: usize) -> Self {
        self.queue_capacity = capacity;
        self
    }

    /// Select the backend.
    pub fn backend(mut self, backend: WatchBackend) -> Self {
        self.backend = backend;
        self
    }

    /// Returns the configured queue capacity.
    pub fn get_queue_capacity(&self) -> usize {
        self.queue_capacity
    }

    /// Returns the configured backend.
    pub fn get_backend(&self) -> WatchBackend {
        self.backend
    }

    /// Reject values that cannot be honored.
    pub(super) fn validate(self) -> Result<Self, WatchError> {
        if self.queue_capacity == 0 {
            return Err(WatchError::InvalidOptions("queue capacity must be at least 1"));
        }
        if let WatchBackend::Polling { interval } = self.backend
            && interval < MIN_POLL_INTERVAL
        {
            return Err(WatchError::InvalidOptions("polling interval is below the minimum"));
        }
        Ok(self)
    }
}
