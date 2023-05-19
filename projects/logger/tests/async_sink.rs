use std::{
    sync::{Arc, Mutex, mpsc},
    time::Duration,
};

use logger::{
    AsyncSink, AsyncSinkError, FlushOutcome, Level, LogEvent, LogSink, VecSink, clear_global_sink, flush_global_sink, set_global_sink,
};
use serial_test::serial;

fn event(target: &str) -> LogEvent {
    LogEvent::log(Level::Info, target).build()
}

struct SharedSink(Arc<Mutex<VecSink>>);

impl LogSink for SharedSink {
    fn emit(&mut self, event: &LogEvent) {
        self.0.lock().expect("vec sink mutex poisoned").emit(event);
    }
}

struct BlockingSink {
    entered: mpsc::SyncSender<()>,
    release: Mutex<mpsc::Receiver<()>>,
}

impl LogSink for BlockingSink {
    fn emit(&mut self, _event: &LogEvent) {
        let _ = self.entered.send(());
        let _ = self.release.lock().expect("release mutex poisoned").recv();
    }
}

#[test]
fn queued_events_reach_the_inner_sink_before_flush_returns() {
    let shared = Arc::new(Mutex::new(VecSink::new()));
    let mut sink = AsyncSink::new(Box::new(SharedSink(shared.clone())), 4).expect("the worker starts");

    sink.emit(&event("worker"));

    assert_eq!(sink.flush(), FlushOutcome::Flushed);
    assert_eq!(sink.dropped(), 0);
    assert_eq!(shared.lock().expect("vec sink mutex poisoned").events().len(), 1);
    assert_eq!(sink.close(), FlushOutcome::Flushed);
}

#[test]
fn a_full_queue_drops_events_and_counts_them() {
    let (entered_sender, entered_receiver) = mpsc::sync_channel(1);
    let (release_sender, release_receiver) = mpsc::channel();
    let blocking = BlockingSink { entered: entered_sender, release: Mutex::new(release_receiver) };
    let mut sink = AsyncSink::new(Box::new(blocking), 1).expect("the worker starts");

    sink.emit(&event("worker"));
    entered_receiver.recv_timeout(Duration::from_secs(5)).expect("the worker picked up the first event");

    sink.emit(&event("worker"));
    sink.emit(&event("worker"));

    assert_eq!(sink.dropped(), 1);

    release_sender.send(()).expect("the worker is waiting");
    release_sender.send(()).expect("the worker is waiting");

    assert_eq!(sink.flush(), FlushOutcome::Flushed);
    assert_eq!(sink.close(), FlushOutcome::Flushed);
}

#[test]
fn close_drains_what_the_queue_still_holds() {
    let shared = Arc::new(Mutex::new(VecSink::new()));
    let mut sink = AsyncSink::new(Box::new(SharedSink(shared.clone())), 8).expect("the worker starts");

    for _ in 0..3 {
        sink.emit(&event("worker"));
    }

    assert_eq!(sink.close(), FlushOutcome::Flushed);
    assert_eq!(shared.lock().expect("vec sink mutex poisoned").events().len(), 3);
}

#[test]
fn an_async_sink_needs_room_for_at_least_one_event() {
    assert!(matches!(AsyncSink::new(Box::new(VecSink::new()), 0), Err(AsyncSinkError::ZeroCapacity)));
}

#[test]
#[serial]
fn the_global_facade_flushes_the_installed_sink() {
    clear_global_sink();
    assert_eq!(flush_global_sink(), None);

    set_global_sink(Box::new(VecSink::new()));
    assert_eq!(flush_global_sink(), Some(FlushOutcome::Flushed));

    clear_global_sink();
}

#[test]
#[serial]
fn installing_again_replaces_the_previous_sink() {
    clear_global_sink();
    let previous = Arc::new(Mutex::new(VecSink::new()));
    let current = Arc::new(Mutex::new(VecSink::new()));

    set_global_sink(Box::new(SharedSink(previous.clone())));
    set_global_sink(Box::new(SharedSink(current.clone())));
    logger::emit(event("worker"));

    assert_eq!(previous.lock().expect("vec sink mutex poisoned").events().len(), 0);
    assert_eq!(current.lock().expect("vec sink mutex poisoned").events().len(), 1);

    clear_global_sink();
}
