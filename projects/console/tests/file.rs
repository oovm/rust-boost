use std::io::{Read, Seek, SeekFrom};

use console::{ConsoleEvent, ConsoleSink, FileSink, Level, VecSink, clear_global_sink, event_to_json, set_global_sink};
use serial_test::serial;

#[test]
fn file_sink_appends_json_lines() {
    let path = std::env::temp_dir().join(format!("console-file-sink-{}.jsonl", std::process::id()));
    let mut sink = FileSink::open(&path).expect("open file sink");
    sink.emit(
        &ConsoleEvent::log(Level::Info, "worker")
            .field("message", console::FieldValue::str("ready"))
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
    let path = std::env::temp_dir().join(format!("console-global-file-{}.jsonl", std::process::id()));
    console::install_global_file_sink(&path).expect("install global file sink");

    let event = ConsoleEvent::log(Level::Warn, "cli")
        .field("message", console::FieldValue::str("blocked"))
        .build();
    let expected = event_to_json(&event);
    console::emit(event);

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
    console::emit(ConsoleEvent::log(Level::Error, "test").field("message", console::FieldValue::str("x")).build());
    clear_global_sink();
}
