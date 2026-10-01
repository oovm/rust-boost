use std::io::Error;

use diagnostic::DiagnosticSeverity;

use crate::{IOError, QError, QErrorKind};

impl From<Error> for QError {
    fn from(error: Error) -> Self {
        let io = IOError { message: error.to_string(), file: Default::default() };
        QError { error: Box::new(QErrorKind::IO(io)), severity: DiagnosticSeverity::Error, source: Some(Box::new(error)) }
    }
}
