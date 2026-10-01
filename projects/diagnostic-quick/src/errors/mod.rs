use std::{
    error::Error,
    fmt::{Debug, Display, Formatter},
};

use diagnostic::DiagnosticSeverity;
use source_cache::{SourceID, SourceSpan};

pub mod display;
mod error_io;
mod error_runtime;
mod error_syntax;

/// Result alias for quick error boundaries.
pub type QResult<T = ()> = Result<T, QError>;

/// Quick error used at API boundaries before conversion to structured diagnostics.
#[derive(Debug)]
pub struct QError {
    /// Error payload.
    pub error: Box<QErrorKind>,
    /// Structured severity for downstream renderers.
    pub severity: DiagnosticSeverity,
    /// Optional chained source error.
    pub source: Option<Box<dyn Error>>,
}

/// Quick error variants.
#[derive(Debug)]
pub enum QErrorKind {
    /// I/O failure.
    IO(IOError),
    /// Syntax failure with optional span.
    Syntax(SyntaxError),
    /// Runtime failure without source span.
    Runtime(RuntimeError),
    /// Custom message.
    Custom(String),
}

/// Syntax error with source span metadata.
#[derive(Debug)]
pub struct SyntaxError {
    /// Human-readable message.
    pub message: String,
    /// Cached source identifier.
    pub file: SourceID,
    /// Byte span in the cached source.
    pub span: SourceSpan,
}

/// Runtime error without location metadata.
#[derive(Debug)]
pub struct RuntimeError {
    /// Human-readable message.
    pub message: String,
}

/// I/O error tied to a cached source identifier.
#[derive(Debug)]
pub struct IOError {
    /// Human-readable message.
    pub message: String,
    /// Cached source identifier when known.
    pub file: SourceID,
}

impl QError {
    /// Create a syntax error without span metadata.
    pub fn syntax_error(msg: impl Into<String>) -> Self {
        let error = SyntaxError { message: msg.into(), file: Default::default(), span: Default::default() };
        Self {
            error: Box::new(QErrorKind::Syntax(error)),
            severity: DiagnosticSeverity::Error,
            source: None,
        }
    }

    /// Create a runtime error.
    pub fn runtime_error(msg: impl Into<String>) -> Self {
        let error = RuntimeError { message: msg.into() };
        Self {
            error: Box::new(QErrorKind::Runtime(error)),
            severity: DiagnosticSeverity::Error,
            source: None,
        }
    }

    /// Returns the error kind.
    pub fn kind(&self) -> &QErrorKind {
        &*self.error
    }

    /// Attach a cached source identifier.
    pub fn with_file(mut self, file: &SourceID) -> Self {
        match &mut *self.error {
            QErrorKind::IO(value) => value.file = file.clone(),
            QErrorKind::Syntax(value) => value.file = file.clone(),
            QErrorKind::Runtime(_) | QErrorKind::Custom(_) => {}
        }
        self
    }

    /// Attach a byte span to a syntax error.
    pub fn with_span(mut self, span: SourceSpan) -> Self {
        if let QErrorKind::Syntax(value) = &mut *self.error {
            value.span = span;
        }
        self
    }

    /// Override structured severity.
    pub fn with_severity(mut self, severity: DiagnosticSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Returns the primary source identifier when present.
    pub fn source_id(&self) -> Option<&SourceID> {
        match &*self.error {
            QErrorKind::IO(value) => Some(&value.file),
            QErrorKind::Syntax(value) if value.file != SourceID::default() => Some(&value.file),
            _ => None,
        }
    }
}

impl Display for QError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl Display for QErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

impl Error for QError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.source {
            Some(source) => Some(&**source),
            None => None,
        }
    }
}
