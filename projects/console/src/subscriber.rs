use logger::set_global_sink;

use crate::StderrSubscriberSink;

/// Install the default human-visible stderr subscriber on the global logger facade.
///
/// Events are rendered through [`StderrSubscriberSink`] using [`render`].
pub fn install_global_subscriber() {
    set_global_sink(Box::new(StderrSubscriberSink));
}
