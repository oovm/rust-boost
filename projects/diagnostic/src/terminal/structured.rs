use alloc::string::{String, ToString};
use core::fmt::{self, Display, Formatter};
use core::ops::Range;

use source_cache::{SourceCache, SourceID, SourceSpan};

use crate::{
    model::Diagnostic,
    ByteRange, DiagnosticLabel, DiagnosticLocation, DiagnosticSeverity, DiagnosticSet, Message, SourceRef,
};

use super::{
    source_map::SourceRegistry,
    Config, Diagnostic as TerminalDiagnostic, Label, ReportKind,
};

/// Failure while converting a structured diagnostic for terminal rendering.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StructuredRenderError {
    /// A byte range could not be converted to a terminal span.
    RangeOutOfBounds {
        /// Inclusive start offset.
        start: u64,
        /// Exclusive end offset.
        end: u64,
    },
    /// A source reference was not registered for text rendering.
    UnknownSource {
        /// Stable wire identifier for the missing source.
        wire_id: String,
    },
    /// Terminal output failed.
    Io(String),
}

impl Display for StructuredRenderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::RangeOutOfBounds { start, end } => {
                write!(f, "byte range [{start}, {end}) is out of bounds for terminal spans")
            }
            Self::UnknownSource { wire_id } => write!(f, "source reference {wire_id} is not registered"),
            Self::Io(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for StructuredRenderError {}

/// Convert a structured diagnostic into the legacy terminal renderer model.
pub fn structured_to_terminal(
    diagnostic: &Diagnostic,
    registry: &SourceRegistry,
    config: Config,
) -> Result<TerminalDiagnostic, StructuredRenderError> {
    let mut builder = TerminalDiagnostic::new(severity_to_kind(diagnostic.severity()))
        .with_message(render_message(diagnostic.message()))
        .with_config(config);

    let mut notes = diagnostic.notes().iter().map(render_message).collect::<Vec<_>>();
    let helps = diagnostic.helps().iter().map(render_message).collect::<Vec<_>>();

    if let Some(primary) = diagnostic.primary() {
        match try_text_label(primary, registry)? {
            Some(terminal_label) => {
                if let Some((file, start)) = primary_text_location(primary.location(), registry) {
                    builder = builder.with_location(file, Some(start));
                }
                builder = builder.with_label(terminal_label);
            }
            None => notes.push(location_summary(primary.location(), primary.message())),
        }
    }

    for secondary in diagnostic.secondary() {
        match try_text_label(secondary, registry)? {
            Some(terminal_label) => builder = builder.with_label(terminal_label),
            None => notes.push(location_summary(secondary.location(), secondary.message())),
        }
    }

    if !notes.is_empty() {
        builder = builder.with_note(notes.join("\n"));
    }
    if !helps.is_empty() {
        builder = builder.with_help(helps.join("\n"));
    }

    Ok(builder.finish())
}

/// Render a structured diagnostic set to `stderr`.
pub fn eprint_structured_set(
    cache: &SourceCache,
    registry: &SourceRegistry,
    set: &DiagnosticSet,
    config: Config,
) -> Result<(), StructuredRenderError> {
    for diagnostic in set.diagnostics() {
        structured_to_terminal(diagnostic, registry, config)?
            .eprint(cache)
            .map_err(|error| StructuredRenderError::Io(error.to_string()))?;
    }
    Ok(())
}

fn severity_to_kind(severity: DiagnosticSeverity) -> ReportKind {
    match severity {
        DiagnosticSeverity::Bug => ReportKind::Fatal,
        DiagnosticSeverity::Error => ReportKind::Error,
        DiagnosticSeverity::Warning => ReportKind::Alert,
        DiagnosticSeverity::Info => ReportKind::Trace,
        DiagnosticSeverity::Hint => ReportKind::Blame,
    }
}

fn render_message(message: &Message) -> String {
    message.fallback().map(ToString::to_string).unwrap_or_else(|| message.key().to_string())
}

fn primary_text_location(location: &DiagnosticLocation, registry: &SourceRegistry) -> Option<(SourceID, u32)> {
    let (source, range) = text_like_location(location)?;
    let file = *registry.resolve(source)?;
    let start = u32::try_from(range.start()).ok()?;
    Some((file, start))
}

fn try_text_label(label: &DiagnosticLabel, registry: &SourceRegistry) -> Result<Option<Label>, StructuredRenderError> {
    if let Some((source, range)) = text_like_location(label.location()) {
        let file = registry
            .resolve(source)
            .ok_or_else(|| StructuredRenderError::UnknownSource { wire_id: source.wire_id() })?;
        let span = byte_range_to_span(*file, range)?;
        return Ok(Some(Label::new(span).with_message(render_message(label.message()))));
    }
    Ok(None)
}

fn text_like_location(location: &DiagnosticLocation) -> Option<(&SourceRef, ByteRange)> {
    match location {
        DiagnosticLocation::Text { source, range } => Some((source, *range)),
        DiagnosticLocation::Virtual { source, range } => Some((source, *range)),
        _ => None,
    }
}

fn byte_range_to_span(file: SourceID, range: ByteRange) -> Result<SourceSpan, StructuredRenderError> {
    let start = u32::try_from(range.start())
        .map_err(|_| StructuredRenderError::RangeOutOfBounds { start: range.start(), end: range.end() })?;
    let end = u32::try_from(range.end())
        .map_err(|_| StructuredRenderError::RangeOutOfBounds { start: range.start(), end: range.end() })?;
    Ok(file.with_range(Range { start, end }))
}

fn location_summary(location: &DiagnosticLocation, message: &Message) -> String {
    let prefix = match location {
        DiagnosticLocation::Binary { source, address_space, range, .. } => {
            format!("{}:{} [{}, {})", source.id(), address_space.id(), range.start(), range.end())
        }
        DiagnosticLocation::Member { container, member, range, precision, .. } => {
            let path = member
                .segments()
                .iter()
                .map(|segment| format!("{}/{}", segment.kind(), segment.name()))
                .collect::<Vec<_>>()
                .join(" -> ");
            let range = range
                .map(|range| format!(" [{}, {})", range.start(), range.end()))
                .unwrap_or_default();
            format!("{}::{path}{range} ({precision:?})", container.id())
        }
        DiagnosticLocation::Object { source, object, .. } => {
            format!("{} {}/{}", source.id(), object.kind(), object.id())
        }
        DiagnosticLocation::Semantic { document, path, .. } => {
            format!("{}:{}", document.id(), path.as_str())
        }
        DiagnosticLocation::Text { source, range } | DiagnosticLocation::Virtual { source, range } => {
            format!("{} [{}, {})", source.id(), range.start(), range.end())
        }
    };
    format!("{prefix}: {}", render_message(message))
}
