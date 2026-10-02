use logger::set_global_sink;

use crate::StderrFallbackSink;

/// Install the default human-visible stderr subscriber on the global logger facade.
///
/// Events are rendered through [`StderrFallbackSink`] until a styled subscriber replaces it.
pub fn install_global_subscriber() {
    set_global_sink(Box::new(StderrFallbackSink));
}
