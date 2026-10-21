//! Unified entry for the boost standard-library enhancement family.
//!
//! Enable capabilities through crate features. Each module re-exports the
//! corresponding `boost-*` crate without copying types or adding global state.

#![deny(missing_docs)]

#[cfg(feature = "console")]
pub mod console {
    //! Human-visible console output, refresh, progress, and log subscribers.
    pub use boost_console::*;
}

#[cfg(feature = "diagnostic")]
pub mod diagnostic {
    //! Structured diagnostic data model and projections.
    pub use boost_diagnostic::*;
}

#[cfg(feature = "fmt")]
pub mod fmt {
    //! Text styling, width, and layout.
    pub use boost_console::fmt::*;
}

#[cfg(feature = "fs")]
pub mod fs {
    //! File system utilities.
    pub use boost_fs::*;
}

#[cfg(feature = "log")]
pub mod log {
    //! Structured log events, spans, filters, and sinks.
    pub use boost_log::*;
}

#[cfg(feature = "random")]
pub mod random {
    //! Random sources, sampling, and distributions.
    pub use boost_random::*;
}

#[cfg(feature = "source")]
pub mod source {
    //! Source identity, snapshots, bytes, and coordinates.
    pub use boost_source::*;
}

#[cfg(feature = "terminal")]
pub mod terminal {
    //! Terminal device capabilities.
    pub use boost_terminal::*;
}

#[cfg(feature = "test")]
pub mod test {
    //! Correctness tests, cases, fixtures, and assertions.
    pub use boost_test::*;
}
