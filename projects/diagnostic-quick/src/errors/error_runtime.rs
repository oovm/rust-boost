use std::{env::VarError, error::Error, sync::PoisonError};

use diagnostic::DiagnosticSeverity;

use crate::{QError, QErrorKind, RuntimeError};

impl<E> From<&E> for RuntimeError
where
    E: Error,
{
    fn from(error: &E) -> Self {
        RuntimeError { message: error.to_string() }
    }
}

impl QError {
    pub(crate) fn wrap_runtime_error<E: Error + 'static>(error: E) -> Self {
        QError {
            error: Box::new(QErrorKind::Runtime(RuntimeError::from(&error))),
            severity: DiagnosticSeverity::Error,
            source: Some(Box::new(error)),
        }
    }
}

impl From<std::fmt::Error> for QError {
    fn from(error: std::fmt::Error) -> Self {
        QError::wrap_runtime_error(error)
    }
}

impl<T> From<PoisonError<T>> for QError {
    fn from(error: PoisonError<T>) -> Self {
        QError {
            error: Box::new(QErrorKind::Runtime(RuntimeError::from(&error))),
            severity: DiagnosticSeverity::Error,
            source: None,
        }
    }
}

impl From<VarError> for QError {
    fn from(error: VarError) -> Self {
        QError::wrap_runtime_error(error)
    }
}

impl From<()> for QError {
    fn from(_: ()) -> Self {
        QError::runtime_error("unreachable unit error")
    }
}
