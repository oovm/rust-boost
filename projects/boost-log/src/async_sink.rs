use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    io,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
        mpsc::{self, Receiver, SyncSender},
    },
    thread::{self, JoinHandle},
};

use crate::{
    event::LogEvent,
    sink::{FlushOutcome, LogSink},
};

/// A bounded background backend for a [`LogSink`].
///
/// Events are handed to a worker thread through a queue with a fixed capacity. When the queue is
/// full the event is dropped and counted instead of blocking the emitting thread, so a slow sink
/// cannot stall the caller. [`AsyncSink::flush`] waits until the worker has flushed everything
/// queued before it, [`AsyncSink::dropped`] reports how many events the queue rejected, and
/// [`AsyncSink::close`] drains the queue and returns the final flush outcome.
pub struct AsyncSink {
    sender: SyncSender<Command>,
    dropped: Arc<AtomicU64>,
    worker: Option<JoinHandle<FlushOutcome>>,
}

enum Command {
    Event(LogEvent),
    Flush(SyncSender<FlushOutcome>),
}

impl AsyncSink {
    /// Start a worker around `sink` with room for `capacity` queued events.
    ///
    /// A capacity of zero is rejected because a rendezvous channel would block the emitting
    /// thread instead of dropping.
    pub fn new(sink: Box<dyn LogSink>, capacity: usize) -> Result<Self, AsyncSinkError> {
        if capacity == 0 {
            return Err(AsyncSinkError::ZeroCapacity);
        }
        let (sender, receiver) = mpsc::sync_channel(capacity);
        let dropped = Arc::new(AtomicU64::new(0));
        let worker =
            thread::Builder::new().name("logger-backend".to_string()).spawn(move || run(sink, receiver)).map_err(AsyncSinkError::Spawn)?;
        Ok(Self { sender, dropped, worker: Some(worker) })
    }

    /// Returns how many events were dropped because the queue was full or closed.
    pub fn dropped(&self) -> u64 {
        self.dropped.load(Ordering::Relaxed)
    }

    /// Drain the queue, flush the inner sink, and stop the worker.
    ///
    /// Dropping the sink without calling this also drains the queue, but the final flush outcome
    /// is not reported then.
    pub fn close(mut self) -> FlushOutcome {
        let worker = self.worker.take();
        drop(self.sender);
        match worker {
            Some(worker) => worker.join().unwrap_or(FlushOutcome::Failed),
            None => FlushOutcome::Flushed,
        }
    }
}

impl LogSink for AsyncSink {
    fn emit(&mut self, event: &LogEvent) {
        if self.sender.try_send(Command::Event(event.clone())).is_err() {
            self.dropped.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn flush(&mut self) -> FlushOutcome {
        let (ack, outcome) = mpsc::sync_channel(0);
        if self.sender.send(Command::Flush(ack)).is_err() {
            return FlushOutcome::Closed;
        }
        outcome.recv().unwrap_or(FlushOutcome::Closed)
    }
}

fn run(mut sink: Box<dyn LogSink>, receiver: Receiver<Command>) -> FlushOutcome {
    while let Ok(command) = receiver.recv() {
        match command {
            Command::Event(event) => sink.emit(&event),
            Command::Flush(ack) => {
                let outcome = sink.flush();
                let _ = ack.send(outcome);
            }
        }
    }
    sink.flush()
}

/// Error returned when an [`AsyncSink`] cannot start.
#[derive(Debug)]
pub enum AsyncSinkError {
    /// The requested queue capacity was zero.
    ZeroCapacity,
    /// The worker thread could not be started.
    Spawn(io::Error),
}

impl Display for AsyncSinkError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::ZeroCapacity => f.write_str("an async sink needs room for at least one queued event"),
            Self::Spawn(error) => write!(f, "the async sink worker could not start: {error}"),
        }
    }
}

impl Error for AsyncSinkError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ZeroCapacity => None,
            Self::Spawn(error) => Some(error),
        }
    }
}
