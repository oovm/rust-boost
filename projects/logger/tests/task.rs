use std::sync::{Arc, Mutex};

use logger::{
    FieldValue, Level, LogEvent, LogSink, TaskContext, VecSink, clear_global_sink, enter, event, event_to_json, reset_dropped_events,
    set_global_filter, set_global_sink, span, with_scope,
};
use serial_test::serial;

struct SharedSink(Arc<Mutex<VecSink>>);

impl LogSink for SharedSink {
    fn emit(&mut self, event: &LogEvent) {
        self.0.lock().expect("vec sink mutex poisoned").emit(event);
    }
}

#[test]
fn a_child_task_records_its_parent() {
    let root = TaskContext::new("compile");
    let child = root.child("lower");

    let event = child.log(Level::Info, "compile.lower").build();
    let task = event.task().expect("the child task is recorded");

    assert_eq!(task.id(), child.id());
    assert_eq!(task.name(), "lower");
    assert_eq!(task.parent(), Some(root.id()));
}

#[test]
fn task_fields_are_attached_to_the_event() {
    let context = TaskContext::new("compile").field("project", FieldValue::str("vmz"));

    let event = context.log(Level::Info, "compile").build();

    assert!(
        event.fields().iter().any(|field| field.name() == "project" && *field.value() == FieldValue::str("vmz")),
        "the task field is carried by the event"
    );
}

#[test]
fn a_child_task_starts_with_the_fields_of_its_parent() {
    let root = TaskContext::new("compile").field("project", FieldValue::str("vmz"));
    let child = root.child("lower");

    assert!(child.fields().iter().any(|field| field.name() == "project"), "the child inherits the context it was started from");
}

/// A thread scope belongs to the thread, so it must not be recorded as the task: an async task can
/// resume on another thread, and the thread scope would then describe the wrong task.
#[test]
#[serial]
fn a_thread_scope_is_not_a_task_context() {
    clear_global_sink();
    reset_dropped_events();
    let sink = Arc::new(Mutex::new(VecSink::new()));
    set_global_sink(Box::new(SharedSink(sink.clone())));
    set_global_filter(logger::Filter::new(Level::Trace));

    let scope = [logger::Field::new("request_id", FieldValue::str("abc"))];
    with_scope(&scope, || {
        let _guard = enter(span!("worker"));
        event!(Info, "worker", message = "inside a thread scope");
    });

    let guard = sink.lock().expect("vec sink mutex poisoned");
    let event = &guard.events()[0];
    assert!(event.task().is_none(), "a thread scope does not become a task");
    assert!(event.fields().iter().any(|field| field.name() == "request_id"), "the thread scope still contributes its fields");
    clear_global_sink();
}

/// Two tasks that interleave on one thread keep their own identity, and an entered span does not
/// attach the task that entered it to the other task's events.
#[test]
fn interleaved_tasks_on_one_thread_keep_their_own_identity() {
    let first = TaskContext::new("first");
    let second = TaskContext::new("second");

    let mut events = Vec::new();
    for step in 0..4u64 {
        let context = if step % 2 == 0 { &first } else { &second };
        let _guard = enter(span!("step"));
        events.push(context.log(Level::Info, "step").field("step", FieldValue::U64(step)).build());
    }

    for (index, event) in events.iter().enumerate() {
        let task = event.task().expect("every event names its task");
        let expected = if index % 2 == 0 { first.id() } else { second.id() };
        assert_eq!(task.id(), expected, "event {index} was attributed to the wrong task");
    }
}

/// A context carried into another thread still names the task it was started for.
#[test]
fn a_task_that_moves_to_another_thread_keeps_its_identity() {
    let context = TaskContext::new("moved");

    let (sender, receiver) = std::sync::mpsc::channel();
    let carried = context.clone();
    std::thread::spawn(move || {
        sender.send(carried.log(Level::Info, "moved").build()).expect("the event is delivered");
    })
    .join()
    .expect("the thread finishes");

    let event = receiver.recv().expect("the event arrives");
    assert_eq!(event.task().expect("the task is recorded").id(), context.id());
}

#[test]
fn the_event_json_names_the_task() {
    let root = TaskContext::new("compile");
    let child = root.child("lower");

    let json = event_to_json(&child.log(Level::Info, "compile.lower").build());

    assert!(json.contains(&format!("\"taskId\":{}", child.id().as_u64())), "the task id is serialized: {json}");
    assert!(json.contains("\"taskName\":\"lower\""), "the task name is serialized: {json}");
    assert!(json.contains(&format!("\"parentTaskId\":{}", root.id().as_u64())), "the parent task id is serialized: {json}");
}

#[test]
fn an_event_without_a_task_carries_no_task_keys() {
    let json = event_to_json(&LogEvent::log(Level::Info, "plain").build());

    assert!(!json.contains("taskId"), "an event without a task has no task identity: {json}");
}

#[test]
#[serial]
fn emit_through_the_global_facade_records_the_task() {
    clear_global_sink();
    reset_dropped_events();
    let sink = Arc::new(Mutex::new(VecSink::new()));
    set_global_sink(Box::new(SharedSink(sink.clone())));
    set_global_filter(logger::Filter::new(Level::Trace));

    let context = TaskContext::new("facade");
    logger::emit(context.log(Level::Info, "facade").build());

    let guard = sink.lock().expect("vec sink mutex poisoned");
    let task = guard.events()[0].task().expect("the task survives the facade");
    assert_eq!(task.id(), context.id());
    clear_global_sink();
}
