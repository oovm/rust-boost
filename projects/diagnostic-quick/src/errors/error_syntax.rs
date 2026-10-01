use std::{
    error::Error,
    num::{ParseFloatError, ParseIntError},
    str::Utf8Error,
};

use diagnostic::DiagnosticSeverity;

use crate::{QError, QErrorKind, SyntaxError};

impl<E> From<&E> for SyntaxError
where
    E: Error,
{
    fn from(error: &E) -> Self {
        SyntaxError { message: error.to_string(), file: Default::default(), span: Default::default() }
    }
}

impl QError {
    pub(crate) fn wrap_syntax_error<E: Error + 'static>(error: E) -> Self {
        QError {
            error: Box::new(QErrorKind::Syntax(SyntaxError::from(&error))),
            severity: DiagnosticSeverity::Error,
            source: Some(Box::new(error)),
        }
    }
}

impl From<Utf8Error> for QError {
    fn from(error: Utf8Error) -> Self {
        QError::wrap_syntax_error(error)
    }
}

impl From<ParseIntError> for QError {
    fn from(error: ParseIntError) -> Self {
        QError::wrap_syntax_error(error)
    }
}

impl From<ParseFloatError> for QError {
    fn from(error: ParseFloatError) -> Self {
        QError::wrap_syntax_error(error)
    }
}
