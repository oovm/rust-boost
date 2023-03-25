use std::sync::{Arc, Mutex};

use logger::{LogEvent, LogSink, VecSink, clear_global_sink, event, set_global_sink};
use serial_test::serial;

use console::install_global_subscriber;

struct SharedSink(Arc<Mutex<VecSink>>);

impl LogSink for SharedSink {
    fn emit(&mut self, event: &LogEvent) {
        self.0.lock().expect("vec sink mutex poisoned").emit(event);
    }
}

#[test]
#[serial]
fn install_global_subscriber_replaces_sink() {
    clear_global_sink();
    let sink = Arc::new(Mutex::new(VecSink::new()));
    set_global_sink(Box::new(SharedSink(sink.clone())));

    install_global_subscriber();
    event!(Info, "console.subscriber", message = "visible");

    let guard = sink.lock().expect("vec sink mutex poisoned");
    assert!(guard.events().is_empty(), "stderr subscriber should replace the test sink");
    clear_global_sink();
}
