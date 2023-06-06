use std::io::{self, Write};

use crate::{LogEvent, LogSink};

use super::render::{render, render_plain};

/// Plain-text stderr sink without terminal styling.
pub struct StderrFallbackSink;

impl LogSink for StderrFallbackSink {
    fn emit(&mut self, event: &LogEvent) {
        let _ = writeln!(io::stderr(), "{}", render_plain(event));
    }
}

/// Default human-visible stderr subscriber using styled [`render`].
pub struct StderrSubscriberSink;

impl LogSink for StderrSubscriberSink {
    fn emit(&mut self, event: &LogEvent) {
        let _ = writeln!(io::stderr(), "{}", render(event));
    }
}
