//! Explicit task identity for events.
//!
//! A task context is a value the caller threads through its own code. It is deliberately not kept in a
//! thread-local: an asynchronous task can resume on another thread, so a thread-local would attribute
//! its events to whichever task last ran on that thread. A thread-local scope therefore stays a thread
//! scope, and only a [`TaskContext`] carried by the caller records a task.

use std::sync::atomic::{AtomicU64, Ordering};

use crate::{
    event::{LogEvent, LogEventBuilder},
    field::{Field, Fields},
    level::Level,
};

static NEXT_TASK_ID: AtomicU64 = AtomicU64::new(1);

/// Identifier for one logical task.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TaskId(u64);

impl TaskId {
    fn next() -> Self {
        Self(NEXT_TASK_ID.fetch_add(1, Ordering::Relaxed))
    }

    /// Returns the raw task identifier.
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

/// Task identity recorded on an event.
///
/// The reference is owned by the event, so a sink or a serialized line still names the task after the
/// task has finished.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskRef {
    id: TaskId,
    name: String,
    parent: Option<TaskId>,
}

impl TaskRef {
    /// Returns the task identifier.
    pub fn id(&self) -> TaskId {
        self.id
    }

    /// Returns the task name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the parent task identifier when this task was started from another task.
    pub fn parent(&self) -> Option<TaskId> {
        self.parent
    }
}

/// Identity and shared fields of one logical task.
///
/// The caller creates a context and passes it to the code that emits events. Nothing about it is
/// stored in a thread-local, so a task that moves between threads keeps its own identity and cannot
/// pick up the identity of another task that ran on the same thread.
#[derive(Clone, Debug, PartialEq)]
pub struct TaskContext {
    id: TaskId,
    name: String,
    parent: Option<TaskId>,
    fields: Fields,
}

impl TaskContext {
    /// Start a root task.
    pub fn new(name: impl Into<String>) -> Self {
        Self { id: TaskId::next(), name: name.into(), parent: None, fields: Fields::new() }
    }

    /// Start a task that descends from this one.
    ///
    /// The child records this task as its parent and starts with this task's fields, so a subtree of
    /// tasks shares the context it was started with. Inheriting is explicit: it only happens through
    /// the context that the caller passes.
    pub fn child(&self, name: impl Into<String>) -> Self {
        Self { id: TaskId::next(), name: name.into(), parent: Some(self.id), fields: self.fields.clone() }
    }

    /// Attach a field to the task and everything started from it.
    pub fn field(mut self, name: impl Into<String>, value: crate::field::FieldValue) -> Self {
        self.fields.push(name, value);
        self
    }

    /// Returns the task identifier.
    pub fn id(&self) -> TaskId {
        self.id
    }

    /// Returns the task name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the parent task identifier when present.
    pub fn parent(&self) -> Option<TaskId> {
        self.parent
    }

    /// Returns the fields shared by this task and its children.
    pub fn fields(&self) -> &[Field] {
        self.fields.as_slice()
    }

    /// Returns the identity that an event records.
    pub fn reference(&self) -> TaskRef {
        TaskRef { id: self.id, name: self.name.clone(), parent: self.parent }
    }

    /// Build an event that belongs to this task.
    ///
    /// A task that can resume on another thread emits through this builder instead of entering a span,
    /// because the span stack is a thread scope and would follow the thread rather than the task.
    pub fn log(&self, level: Level, target: impl Into<String>) -> LogEventBuilder {
        LogEvent::log(level, target).task(self)
    }
}
