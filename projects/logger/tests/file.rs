use std::io::{Read, Seek, SeekFrom};

use logger::{
    FieldValue, FileSink, Level, LogEvent, LogSink, VecSink, clear_global_sink, emit, event_to_json,
    install_global_file_sink, set_global_sink,
};
use serial_test::serial;

#[test]
fn file_sink_appends_json_lines() {
    let path = std::env::temp_dir().join(format!("logger-file-sink-{}.jsonl", std::process::id()));
    let mut sink = FileSink::open(&path).expect("open file sink");
    sink.emit(
        &LogEvent::log(Level::Info, "worker")
            .field("message", FieldValue::str("ready"))
            .build(),
    );
    sink.flush().expect("flush file sink");

    let mut file = std::fs::File::open(&path).expect("reopen output");
    let mut text = String::new();
    file.read_to_string(&mut text).expect("read output");
    assert!(text.contains("\"kind\":\"log\""));
    assert!(text.ends_with('\n'));

    let _ = std::fs::remove_file(path);
}

#[test]
#[serial]
fn install_global_file_sink_routes_events() {
    let path = std::env::temp_dir().join(format!("logger-global-file-{}.jsonl", std::process::id()));
    install_global_file_sink(&path).expect("install global file sink");

    let event = LogEvent::log(Level::Warn, "cli")
        .field("message", FieldValue::str("blocked"))
        .build();
    let expected = event_to_json(&event);
    emit(event);

    clear_global_sink();

    let mut file = std::fs::File::open(&path).expect("reopen output");
    file.seek(SeekFrom::Start(0)).expect("rewind output");
    let mut text = String::new();
    file.read_to_string(&mut text).expect("read output");
    assert_eq!(text, format!("{expected}\n"));

    let _ = std::fs::remove_file(path);
}

#[test]
#[serial]
fn vec_sink_still_collects_for_tests() {
    let sink = VecSink::new();
    set_global_sink(Box::new(sink));
    emit(LogEvent::log(Level::Error, "test").field("message", FieldValue::str("x")).build());
    clear_global_sink();
}
